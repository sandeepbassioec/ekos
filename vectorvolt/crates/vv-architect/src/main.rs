//! `vv-architect` — Well-Architected Analyzer.
//!
//! Phase 1: `vv-architect scan-code <dir>` — language-agnostic entity + FK
//! detection over a source tree, producing a human report and a
//! `vectorvolt-blueprint.toml` machine-readable blueprint.
//!
//! Later phases (not yet implemented — see the implementation plan's
//! VV-ARCHITECT section): `--scan-db` schema introspection, OpenAPI/Proto
//! schema scanning, the CPU/RAM/shard sizing rule engine, and event/
//! projection detection. This binary is intentionally separate from `vvctl`
//! — it analyzes an arbitrary EXTERNAL system, not a running VectorVolt
//! instance.

mod blueprint;
mod report;
mod scanner;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use blueprint::Blueprint;

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
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::ScanCode { dir, output } => {
            let scan = scanner::scan_directory(&dir)
                .with_context(|| format!("scanning {}", dir.display()))?;
            let bp = Blueprint::from_scan(&scan);

            println!("{}", report::render(&bp));

            let toml = bp.to_toml()?;
            std::fs::write(&output, &toml)
                .with_context(|| format!("writing {}", output.display()))?;
            println!("Blueprint written to {}", output.display());
        }
    }
    Ok(())
}
