use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
use commands::{store, table, wal};
use store::OutputFormat;

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

    /// Output format.
    #[arg(long, global = true, value_enum, default_value = "text")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Table read/write commands (Phase 2 engine).
    #[command(subcommand)]
    Table(table::TableCommand),

    /// KV store read/write commands.
    #[command(subcommand)]
    Store(StoreCommand),

    /// WAL inspection and verification commands.
    #[command(subcommand)]
    Wal(wal::WalCommand),

    /// Print version and build info.
    Version,
}

#[derive(Subcommand)]
enum StoreCommand {
    /// Write a key-value pair (WAL-durable).
    Put {
        key: String,
        value: String,
        /// Store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Look up a single key.
    Get {
        key: String,
        /// Store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Delete a key (writes a tombstone).
    Delete {
        key: String,
        /// Store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Scan a range or prefix of keys.
    Scan {
        /// Store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Return only keys whose prefix matches this string.
        #[arg(long)]
        prefix: Option<String>,
        /// Range start key (inclusive).
        #[arg(long)]
        start: Option<String>,
        /// Range end key (inclusive).
        #[arg(long)]
        end: Option<String>,
        /// Maximum entries to return.
        #[arg(short = 'n', long)]
        limit: Option<usize>,
        /// Return keys only (suppress values).
        #[arg(long)]
        keys_only: bool,
    },

    /// Show store statistics (memtable + SSTable summary).
    Stats {
        /// Store directory.
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

    /// Force flush and compact all SSTables into one.
    Compact {
        /// Store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .with_target(false)
        .compact()
        .init();

    let fmt = cli.output;

    match cli.command {
        Commands::Table(cmd) => table::run(cmd)?,

        Commands::Wal(cmd) => wal::run(cmd)?,

        Commands::Store(cmd) => match cmd {
            StoreCommand::Put { key, value, dir } => store::cmd_put(dir, key, value, fmt)?,
            StoreCommand::Get { key, dir } => store::cmd_get(dir, key, fmt)?,
            StoreCommand::Delete { key, dir } => store::cmd_delete(dir, key, fmt)?,
            StoreCommand::Scan {
                dir,
                prefix,
                start,
                end,
                limit,
                keys_only,
            } => store::cmd_scan(dir, prefix, start, end, limit, keys_only, fmt)?,
            StoreCommand::Stats { dir } => store::cmd_stats(dir, fmt)?,
            StoreCommand::Compact { dir } => store::cmd_compact(dir, fmt)?,
        },

        Commands::Version => print_version(),
    }

    Ok(())
}

fn print_version() {
    println!("vvctl {VERSION}");
    println!("VectorVolt — ultra-low-latency kernel-bypass data engine");
    println!("{HOMEPAGE}");
    println!();
    println!("Build: {} {}", std::env::consts::OS, std::env::consts::ARCH);
}
