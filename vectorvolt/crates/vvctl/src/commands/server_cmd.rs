//! vvctl server sub-commands (status, stop).

use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use super::status_cmd::{self, StatusArgs};
use super::wire::WireOpts;

/// Address of the server to manage.
#[derive(Debug, Args)]
pub struct ServerTarget {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, default_value = "5995")]
    pub port: u16,
    #[command(flatten)]
    pub conn: WireOpts,
}

/// Server management commands.
#[derive(Debug, Subcommand)]
pub enum ServerCommand {
    /// Query a running server over the wire protocol.
    Status(ServerTarget),
    /// Stop the running server (not implemented).
    Stop(ServerTarget),
}

/// Dispatch a `ServerCommand`.
pub fn run(cmd: ServerCommand) -> Result<()> {
    match cmd {
        ServerCommand::Status(t) => status_cmd::run(StatusArgs {
            host: t.host,
            port: t.port,
            conn: t.conn,
        }),
        ServerCommand::Stop(t) => bail!(
            "server stop is not implemented: the wire protocol has no shutdown request; \
             stop the `vvctl serve` process at {}:{} directly (Ctrl-C / service manager)",
            t.host,
            t.port
        ),
    }
}
