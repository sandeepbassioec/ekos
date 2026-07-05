use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
use commands::{store, wal};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HOMEPAGE: &str = "https://workerruntime.com";

#[derive(Parser)]
#[command(
    name    = "vvctl",
    version = VERSION,
    about   = "VectorVolt control plane CLI — workerruntime.com",
    long_about = None,
    propagate_version = true,
)]
struct Cli {
    /// Enable verbose / debug output.
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// WAL inspection and verification commands.
    Wal {
        #[command(subcommand)]
        cmd: wal::WalCommand,
    },

    /// KV store commands.
    Store {
        #[command(subcommand)]
        cmd: StoreCommand,
    },

    /// Print version and build info.
    Version,
}

#[derive(Subcommand)]
enum StoreCommand {
    /// Show store statistics (memtable + SSTable summary).
    Stats {
        /// Path to the store directory.
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Look up a key.
    Get {
        key: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Write a key-value pair.
    Put {
        key: String,
        value: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Delete a key (writes a tombstone).
    Delete {
        key: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Force compaction of all SSTables.
    Compact {
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Wal { cmd } => wal::run(cmd)?,
        Commands::Store { cmd } => match cmd {
            StoreCommand::Stats { dir }               => store::cmd_stats(dir)?,
            StoreCommand::Get { key, dir }            => store::cmd_get(dir, key)?,
            StoreCommand::Put { key, value, dir }     => store::cmd_put(dir, key, value)?,
            StoreCommand::Delete { key, dir }         => store::cmd_delete(dir, key)?,
            StoreCommand::Compact { dir }             => store::cmd_compact(dir)?,
        },
        Commands::Version => print_version(),
    }

    Ok(())
}

fn print_version() {
    println!("vvctl {}", VERSION);
    println!("VectorVolt — ultra-low-latency kernel-bypass data engine");
    println!("{}", HOMEPAGE);
    println!();
    println!("Build: {} {}", std::env::consts::OS, std::env::consts::ARCH);
}
