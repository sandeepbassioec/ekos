//! vvctl server sub-commands (status, stop, logs).

use anyhow::Result;
use clap::Subcommand;

/// Server management commands.
#[derive(Debug, Subcommand)]
pub enum ServerCommand {
    /// Show server status.
    Status,
    /// Stop the running server.
    Stop,
}

/// Dispatch a `ServerCommand`.
pub fn run(cmd: ServerCommand) -> Result<()> {
    match cmd {
        ServerCommand::Status => {
            println!("server: no server running");
            Ok(())
        }
        ServerCommand::Stop => {
            anyhow::bail!("no server running");
        }
    }
}
