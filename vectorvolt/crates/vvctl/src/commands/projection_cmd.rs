//! `vvctl projection` — manage materialized read models (VV-CTL-xxx / S22-L15).

use clap::{Args, Subcommand};

use anyhow::Result;

#[derive(Debug, Args)]
pub struct ProjectionArgs {
    #[command(subcommand)]
    pub command: ProjectionCommand,
}

#[derive(Debug, Subcommand)]
pub enum ProjectionCommand {
    /// List all registered read models.
    List,
    /// Show status and snapshot size of a read model.
    Status {
        /// Read model name.
        name: String,
    },
    /// Trigger WAL replay for a read model (rebuilds from scratch).
    Replay {
        /// Read model name.
        name: String,
    },
    /// Flush the in-process (and Redis) cache for a read model.
    InvalidateCache {
        /// Read model name, or `--all` to flush everything.
        name: String,
    },
}

/// No server API exposes read models (no wire opcode or HTTP route), so every
/// subcommand fails rather than printing placeholder output.
pub fn run(args: ProjectionArgs) -> Result<()> {
    let what = match args.command {
        ProjectionCommand::List => "list".to_string(),
        ProjectionCommand::Status { name } => format!("status {name}"),
        ProjectionCommand::Replay { name } => format!("replay {name}"),
        ProjectionCommand::InvalidateCache { name } => format!("invalidate-cache {name}"),
    };
    anyhow::bail!(
        "projection {what}: not implemented - the server exposes no read-model          management API to query or control"
    )
}
