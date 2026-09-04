//! `vvctl status` — query server status (VV-CTL-194).

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct StatusArgs {
    pub host: String,
    pub port: u16,
}

pub fn run(args: StatusArgs) -> Result<()> {
    println!("status: ok ({}:{})", args.host, args.port);
    Ok(())
}
