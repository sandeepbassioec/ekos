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

pub fn run(args: ProjectionArgs) -> Result<()> {
    match args.command {
        ProjectionCommand::List => {
            println!("{:<30} {:<12} {:<10}", "NAME", "STATUS", "CACHE");
            println!("{}", "-".repeat(55));
            println!("(no live server — start with `vvctl serve` and connect)");
        }
        ProjectionCommand::Status { name } => {
            println!("Read model: {name}");
            println!("Status:     (requires live server)");
        }
        ProjectionCommand::Replay { name } => {
            println!("Triggering WAL replay for read model '{name}' …");
            println!("(requires live server)");
        }
        ProjectionCommand::InvalidateCache { name } => {
            println!("Invalidating cache for read model '{name}' …");
            println!("(requires live server)");
        }
    }
    Ok(())
}
