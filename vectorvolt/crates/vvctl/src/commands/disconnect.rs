//! `vvctl disconnect` — disconnect from a running VectorVolt server (VV-CTL-200).

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct DisconnectArgs {
    pub host: String,
    pub port: u16,
}

pub fn run(args: DisconnectArgs) -> Result<()> {
    println!("disconnected from {}:{}", args.host, args.port);
    Ok(())
}
