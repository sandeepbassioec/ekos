//! `vvctl disconnect` — not implemented (VV-CTL-200).
//!
//! The wire protocol has no session-management or kick opcode, and `vvctl`
//! holds no long-lived connection to close, so there is nothing real to do.
//! It fails loudly instead of printing a fake "disconnected".

use anyhow::{Result, bail};
use clap::Args;

#[derive(Args)]
pub struct DisconnectArgs {
    pub host: String,
    pub port: u16,
}

pub fn run(args: DisconnectArgs) -> Result<()> {
    bail!(
        "disconnect is not implemented: the wire protocol has no session-termination \
         request, and vvctl keeps no persistent connection to {}:{}",
        args.host,
        args.port
    )
}
