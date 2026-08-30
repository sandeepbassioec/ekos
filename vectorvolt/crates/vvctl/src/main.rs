use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use store::OutputFormat;
use vvctl::commands::{
    bench, catalog, config, connect, hal, http, keygen, serve, store, table, tune, wal,
};
use vvctl::ui;

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

    /// Disable all ANSI color codes (also honoured via the NO_COLOR env var).
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress decorative output (banners, spinners, table borders).
    /// Machine-readable values are still printed.
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a VectorVolt table server (opens a table and listens for connections).
    Serve(serve::ServeArgs),

    /// In-process ShardCatalog management — create, list, drop, and describe tables
    /// without needing a running server (VV-CTL-002).
    Catalog(catalog::CatalogArgs),

    /// Master configuration management — init, show, and validate vectorvolt.yaml (VV-CFG-012).
    #[command(subcommand)]
    Config(config::ConfigCommand),

    /// Send a single command to a running VectorVolt server.
    Connect(connect::ConnectArgs),

    /// Table read/write commands (in-process, no server required).
    #[command(subcommand)]
    Table(table::TableCommand),

    /// In-process micro-benchmark: INSERT / GET / SCAN latency.
    Bench(bench::BenchArgs),

    /// Report detected CPU features and measure scalar-vs-SIMD kernel speedup.
    Hal(hal::HalArgs),

    /// KV store read/write commands.
    #[command(subcommand)]
    Store(StoreCommand),

    /// WAL inspection and verification commands.
    #[command(subcommand)]
    Wal(wal::WalCommand),

    /// Send commands to the VectorVolt HTTP gateway (analytics, TTL, batch ops, DSL).
    Http(http::HttpArgs),

    /// Generate a cryptographically random 32-byte hex secret (for use with --secret).
    Keygen(keygen::KeygenArgs),

    /// Detect hardware and derive recommended engine configuration (VV-CFG-006).
    Tune(tune::TuneArgs),

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

    ui::init(cli.no_color, cli.quiet);

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
        Commands::Serve(args) => serve::run(args)?,

        Commands::Catalog(args) => catalog::run(args)?,

        Commands::Config(cmd) => config::run(cmd)?,

        Commands::Connect(args) => connect::run(args)?,

        Commands::Table(cmd) => table::run(cmd)?,

        Commands::Bench(args) => bench::run(args)?,

        Commands::Hal(args) => hal::run(args)?,

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

        Commands::Http(args) => http::run(args)?,

        Commands::Keygen(args) => keygen::run(args)?,

        Commands::Tune(args) => tune::run(args)?,

        Commands::Version => print_version(),
    }

    Ok(())
}

fn print_version() {
    println!("{}", ui::banner());
    println!();
    if !ui::quiet_mode() {
        println!("  {HOMEPAGE}");
        println!(
            "  build: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!("  hal:   {}", vectorvolt_hal::describe());
    }
}
