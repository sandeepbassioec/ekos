//! `vv-architect` — Well-Architected Analyzer.
//!
//! Phase 1: `vv-architect scan-code <dir>` — language-agnostic entity + FK
//! detection over a source tree, producing a human report and a
//! `vectorvolt-blueprint.toml` machine-readable blueprint.
//!
//! Phase 2: `vv-architect scan-db --sqlite <path>` — real foreign-key
//! introspection from an actual SQLite schema (authoritative, not inferred).
//! Postgres/MySQL connection-string scanning is deferred: it needs a live
//! server this environment can't spin up for tests, see db_scanner.rs.
//!
//! Phase 3: `--tps`/`--data-gb` on either scan command additionally runs the
//! cluster sizing rule engine (see rules.rs) and folds its output into the
//! same blueprint — transparent heuristic math over caller-supplied
//! workload estimates, not a claim of profiling a real running system.
//!
//! Later phases (not yet implemented — see the implementation plan's
//! VV-ARCHITECT section): OpenAPI/Proto schema scanning and event/
//! projection detection. This binary is intentionally separate from `vvctl`
//! — it analyzes an arbitrary EXTERNAL system, not a running VectorVolt
//! instance.

mod blueprint;
mod db_scanner;
mod report;
mod rules;
mod scanner;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use blueprint::Blueprint;
use rules::SizingInputs;

#[derive(Parser)]
#[command(
    name = "vv-architect",
    version,
    about = "VectorVolt Well-Architected Analyzer"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Optional workload estimates that additionally trigger the cluster sizing
/// rule engine. Both flags must be given together — sizing needs both an
/// expected write rate and an expected data volume to mean anything.
#[derive(Args, Clone, Copy)]
struct SizingArgs {
    /// Expected peak transactions/sec — enables cluster sizing output.
    #[arg(long, requires = "data_gb")]
    tps: Option<u64>,

    /// Expected total data volume in GB — enables cluster sizing output.
    #[arg(long, requires = "tps")]
    data_gb: Option<f64>,

    /// Replicas per shard in addition to the primary.
    #[arg(long, default_value_t = 2)]
    repl_factor: u32,
}

impl SizingArgs {
    fn inputs(&self) -> Option<SizingInputs> {
        match (self.tps, self.data_gb) {
            (Some(estimated_tps), Some(estimated_data_gb)) => Some(SizingInputs {
                estimated_tps,
                estimated_data_gb,
                replication_factor: self.repl_factor,
            }),
            _ => None,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Scan a source code directory for entities and foreign-key
    /// relationships, and emit a deployment blueprint.
    ScanCode {
        /// Directory to scan recursively.
        dir: PathBuf,

        /// Where to write the machine-readable blueprint TOML.
        #[arg(long, default_value = "vectorvolt-blueprint.toml")]
        output: PathBuf,

        #[command(flatten)]
        sizing: SizingArgs,
    },

    /// Scan a database schema for tables and real foreign-key constraints,
    /// and emit a deployment blueprint. Currently supports SQLite only.
    ScanDb {
        /// Path to a SQLite database file.
        #[arg(long)]
        sqlite: PathBuf,

        /// Where to write the machine-readable blueprint TOML.
        #[arg(long, default_value = "vectorvolt-blueprint.toml")]
        output: PathBuf,

        #[command(flatten)]
        sizing: SizingArgs,
    },
}

fn finish(mut bp: Blueprint, sizing: &SizingArgs, output: &PathBuf) -> Result<()> {
    if let Some(inputs) = sizing.inputs() {
        let hint = bp.suggested_partition_key();
        bp = bp.with_sizing(rules::compute(inputs, hint));
    }

    println!("{}", report::render(&bp));

    let toml = bp.to_toml()?;
    std::fs::write(output, &toml).with_context(|| format!("writing {}", output.display()))?;
    println!("Blueprint written to {}", output.display());
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::ScanCode {
            dir,
            output,
            sizing,
        } => {
            let scan = scanner::scan_directory(&dir)
                .with_context(|| format!("scanning {}", dir.display()))?;
            finish(Blueprint::from_scan(&scan), &sizing, &output)?;
        }
        Command::ScanDb {
            sqlite,
            output,
            sizing,
        } => {
            let scan = db_scanner::scan_sqlite(&sqlite)
                .with_context(|| format!("scanning {}", sqlite.display()))?;
            finish(Blueprint::from_scan(&scan), &sizing, &output)?;
        }
    }
    Ok(())
}
