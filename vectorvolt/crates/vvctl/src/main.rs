use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use store::OutputFormat;
use vvctl::commands::{
    bench, bench_net, catalog, chaos, cluster_cmd, completion, config, config_audit, connect,
    disconnect, events_cmd, graph, hal, http, install, keygen, ledger_cmd, linux, ping, profile,
    projection_cmd, query_cmd, serve, server_cmd, status_cmd, store, support_bundle, table,
    topology, tune, wal, xdp_cmd,
};
use vvctl::ui;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HOMEPAGE: &str = "https://workerruntime.com";

#[derive(Clone, Debug, clap::ValueEnum)]
enum LogFormat {
    Text,
    Json,
}

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

    /// Log format: "text" (human-readable, default) or "json" (structured, for log aggregators).
    #[arg(long, global = true, default_value = "text")]
    log_format: LogFormat,

    /// OTLP gRPC endpoint for distributed tracing (e.g. http://localhost:4317).
    /// Omit to disable OpenTelemetry export (zero overhead).
    #[arg(long, global = true)]
    otel_endpoint: Option<String>,

    /// Service name reported in OpenTelemetry spans (default: vvctl).
    #[arg(long, global = true, default_value = "vvctl")]
    otel_service_name: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a VectorVolt table server (opens a table and listens for connections).
    Serve(Box<serve::ServeArgs>),

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

    /// Emit the host SystemProfile as JSON or a human-readable table (VV-CFG-001).
    Profile(profile::ProfileArgs),

    /// Show detected CPU/NUMA topology.
    Topology(topology::TopologyArgs),

    /// Print shell completion scripts.
    Completion(completion::CompletionArgs),

    /// Server management commands (status, health).
    #[command(subcommand)]
    Server(server_cmd::ServerCommand),

    /// Collect diagnostics into a support bundle file.
    SupportBundle(support_bundle::SupportBundleArgs),

    /// Audit a config file for deprecated keys (VV-CTL-184).
    ConfigAudit {
        /// Path to config file (default: vectorvolt.yaml).
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },

    /// Send N pings to a running VectorVolt server (VV-CTL-191).
    Ping(ping::PingArgs),

    /// Query server status (VV-CTL-194).
    Status(status_cmd::StatusArgs),

    /// Benchmark network request throughput (VV-CTL-198).
    BenchNet(bench_net::BenchNetArgs),

    /// Disconnect from a server (VV-CTL-200).
    Disconnect(disconnect::DisconnectArgs),

    /// Graph engine commands (add/remove edges, traverse, cascade, pagerank).
    #[command(subcommand)]
    Graph(graph::GraphCommand),

    /// Chaos injection and deterministic simulation testing.
    #[command(subcommand)]
    Chaos(chaos::ChaosCommand),

    /// Double-entry ledger commands.
    #[command(subcommand)]
    Ledger(ledger_cmd::LedgerCommand),

    /// Linux-specific hardware diagnostics (NUMA, affinity, hugepages, O_DIRECT, io_uring).
    Linux(linux::LinuxArgs),

    /// AF_XDP kernel-bypass networking commands (stats, bench, umem layout).
    #[command(subcommand)]
    Xdp(xdp_cmd::XdpCommand),

    /// Manage materialized read models (list, status, replay, invalidate-cache) — S22.
    #[command(subcommand)]
    Projection(projection_cmd::ProjectionCommand),

    /// Provision the host: create system user, harden data dir, generate master key (VV-SEC-003).
    Install(install::InstallArgs),

    /// Cluster membership management (nodes, join, leave, status, ring).
    Cluster(cluster_cmd::ClusterArgs),

    /// Domain event schema discovery and DLQ management (list, show, diff) — S23/S24.
    Events(events_cmd::EventsArgs),

    /// Execute ad-hoc SQL against a local table directory (SELECT/INSERT/DELETE/EXPLAIN) — S24-B.
    Query(query_cmd::QueryArgs),

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

    // Init OTel BEFORE the tracing subscriber so the global provider is set first.
    let _otel_guard = vectorvolt_otel::init(cli.otel_endpoint.as_deref(), &cli.otel_service_name);

    let level = if cli.verbose { "debug" } else { "warn" };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));
    let otel_layer = vectorvolt_otel::layer(&_otel_guard);
    match cli.log_format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(otel_layer)
                .with(tracing_subscriber::fmt::layer().json())
                .with(filter)
                .init();
        }
        LogFormat::Text => {
            tracing_subscriber::registry()
                .with(otel_layer)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .compact(),
                )
                .with(filter)
                .init();
        }
    }

    // Completion must be handled before the normal dispatch: clap_complete
    // needs a fresh Command instance built from the Cli type.
    if let Commands::Completion(ref args) = cli.command {
        completion::generate(&args.shell, &mut Cli::command());
        return Ok(());
    }

    let fmt = cli.output;

    match cli.command {
        Commands::Serve(args) => serve::run(*args)?,

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

        Commands::Profile(args) => profile::run(args)?,

        Commands::Topology(args) => topology::run(args)?,

        Commands::Completion(args) => completion::run(args)?,

        Commands::Server(cmd) => server_cmd::run(cmd)?,

        Commands::SupportBundle(args) => support_bundle::run(args)?,

        Commands::ConfigAudit { path } => config_audit::run(path)?,

        Commands::Ping(args) => ping::run(args)?,

        Commands::Status(args) => status_cmd::run(args)?,

        Commands::BenchNet(args) => bench_net::run(args)?,

        Commands::Disconnect(args) => disconnect::run(args)?,

        Commands::Graph(cmd) => graph::run(cmd)?,

        Commands::Chaos(cmd) => chaos::run(cmd)?,

        Commands::Ledger(cmd) => ledger_cmd::run(cmd)?,

        Commands::Linux(args) => linux::run(&args)?,

        Commands::Xdp(cmd) => xdp_cmd::run(cmd)?,

        Commands::Projection(cmd) => {
            projection_cmd::run(projection_cmd::ProjectionArgs { command: cmd })?
        }

        Commands::Install(args) => install::run(args)?,

        Commands::Cluster(args) => cluster_cmd::run(args)?,

        Commands::Events(args) => events_cmd::run(args)?,

        Commands::Query(args) => query_cmd::run(args)?,

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
