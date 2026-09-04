//! `vvctl connect-simple` — open a simple connection to a VectorVolt server (VV-CTL-196).

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ConnectSimpleArgs {
    pub host: String,
    pub port: u16,
}

pub fn run(args: ConnectSimpleArgs) -> Result<()> {
    println!("connecting to {}:{}", args.host, args.port);
    Ok(())
}
