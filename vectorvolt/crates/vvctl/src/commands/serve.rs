//! `vvctl serve` — start a VectorVolt table server.
//!
//! Opens (or creates) a table and listens for VectorVolt wire-protocol
//! connections. The server runs until killed (Ctrl-C / SIGTERM).
//!
//! ## Quickstart
//!
//! ```text
//! # plaintext, no auth
//! vvctl serve ./data --columns pk age score
//!
//! # with a static secret key (hex)
//! vvctl serve ./data --secret aabbccdd...
//!
//! # with TLS
//! vvctl serve ./data --cert server.pem --key server.key
//! ```

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::Args;
use vectorvolt_engine::{
    CliOverrides, ShardCatalog, SystemProfile, TableEngine, TableRegistry, VvConfig,
    load_config_full,
};
use vectorvolt_http::{HttpServer, HttpServerConfig};
use vectorvolt_net::tls::ServerTlsConfig;
use vectorvolt_net::{AuthConfig, Scope, Secret, Server, ServerConfig};

const SCHEMA_FILE: &str = "schema.txt";

#[derive(Args)]
pub struct ServeArgs {
    /// Table directory. Created if it does not exist.
    pub dir: PathBuf,

    /// Column names when creating a new table (first = primary key).
    /// Ignored if the table already exists.
    #[arg(long, num_args = 1..)]
    pub columns: Vec<String>,

    /// Address to listen on.
    #[arg(long, short, default_value = "127.0.0.1:5995")]
    pub addr: SocketAddr,

    /// Maximum concurrent connections.
    #[arg(long, default_value = "256")]
    pub max_connections: usize,

    /// 32-byte shared secret (hex) for HMAC-SHA3 challenge-response auth.
    /// If omitted, authentication is disabled — suitable only for localhost.
    #[arg(long)]
    pub secret: Option<String>,

    /// Key ID label to use with `--secret` (arbitrary string).
    #[arg(long, default_value = "default")]
    pub key_id: String,

    /// PEM certificate file for TLS.
    #[arg(long)]
    pub cert: Option<PathBuf>,

    /// PEM private-key file for TLS.
    #[arg(long)]
    pub key: Option<PathBuf>,

    /// Multi-table mode: `dir` is a catalog root, not a single table directory.
    ///
    /// All subdirectories of `dir` that contain a `schema.txt` are opened as
    /// named tables. New tables can be created at runtime via `vvctl connect
    /// create-table` and will persist across restarts.
    ///
    /// When this flag is set, `--columns` is ignored.
    #[arg(long)]
    pub multi: bool,

    /// Load engine tunables from a TOML or YAML config file (VV-ENG-017).
    ///
    /// Precedence (highest wins): CLI flags > VECTORVOLT_* env vars > config file > host profile.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Load the master multi-section VectorVolt config (VV-CFG-011).
    ///
    /// Accepts a `vectorvolt.yaml` file written with `vvctl catalog config-set`.
    /// Sections applied (CLI flags override):
    /// - `network.bind_addr` → default listen address
    /// - `network.max_connections` → default max connections
    /// - `threads.writer_count` → default partition count
    /// - `storage.l0_flush_threshold` → default flush threshold
    #[arg(long)]
    pub master_config: Option<PathBuf>,

    /// Override flush threshold (rows buffered before WAL flush). Wins over config + env.
    #[arg(long)]
    pub flush_threshold: Option<usize>,

    /// Override durability mode ("fsync" | "write_only" | "buffered"). Wins over config + env.
    #[arg(long)]
    pub durability: Option<String>,

    /// Override max partitions per table. Wins over config + env.
    #[arg(long)]
    pub max_partitions: Option<usize>,

    /// Open a ShardCatalog (TxnManager-backed) alongside the wire server.
    ///
    /// Implies multi-table mode. The catalog is stored at `<dir>/catalog/` and
    /// exposed via `BoundServer.catalog` for direct TxnManager access. The wire
    /// server is bound via `TableRegistry` until S4 adds native catalog binding.
    #[arg(long)]
    pub catalog: bool,

    /// On startup, reload all tables persisted in the schema registry.
    ///
    /// Requires `--catalog`. Calls `ShardCatalog::restore_all()` immediately
    /// after the catalog is opened so tables from prior sessions are available
    /// without an explicit `vvctl catalog restore` call.
    #[arg(long, requires = "catalog")]
    pub restore_catalog: bool,

    /// Also start an HTTP/REST gateway on this address (e.g. `127.0.0.1:7879`).
    ///
    /// The gateway exposes a JSON API that mirrors the wire-protocol opcodes,
    /// making VectorVolt accessible from any HTTP client. Disabled if omitted.
    #[arg(long)]
    pub http_addr: Option<SocketAddr>,

    /// Bearer token required on every HTTP gateway request.
    ///
    /// When set, every non-observability request to the HTTP gateway must
    /// carry `Authorization: Bearer <token>`.  Has no effect when
    /// `--http-addr` is not specified.
    #[arg(long)]
    pub http_token: Option<String>,

    /// Network transport: "tcp" (default) or "xdp" (Linux, requires CAP_NET_RAW).
    ///
    /// XDP bypasses the kernel TCP/IP stack for lower latency on supported NICs.
    /// Requires `--xdp-iface` when set to "xdp".
    #[arg(long, default_value = "tcp")]
    pub transport: String,

    /// Network interface for XDP transport (e.g. "eth0", "lo").
    /// Only used when `--transport xdp` is set.
    #[arg(long, default_value = "lo")]
    pub xdp_iface: String,

    /// NIC queue index for XDP (default 0). Only used with `--transport xdp`.
    #[arg(long, default_value = "0")]
    pub xdp_queue: u32,

    /// XDP ring size (power of two). Only used with `--transport xdp`.
    #[arg(long, default_value = "256")]
    pub xdp_ring_size: u32,

    /// Also start a gRPC write server on this port (default 50051, 0 = disabled).
    ///
    /// Exposes the same insert/delete/query/txn operations as the wire-protocol
    /// TCP server but via gRPC (HTTP/2, protobuf). Useful for integrations with
    /// gRPC-native ecosystems.
    #[arg(long, default_value = "50051")]
    pub grpc_port: u16,

    /// Disable the gRPC server (equivalent to `--grpc-port 0`).
    #[arg(long)]
    pub no_grpc: bool,

    /// Expose a Prometheus `/metrics` endpoint on this port (default 9100, 0 = disabled).
    ///
    /// Serves connections-accepted, active-connections, requests-total, errors-total,
    /// WAL entries, and byte counters in Prometheus text exposition format.
    #[arg(long, default_value = "9100")]
    pub metrics_port: u16,

    /// Unseal the master key from the TPM blob at `<dir>/tpm_sealed.bin` on startup.
    ///
    /// Fails immediately if the TPM is not present, the blob is missing, or the PCR
    /// policy does not match (boot chain tampered). Requires Linux + TPM 2.0 hardware.
    #[arg(long)]
    pub tpm_unseal: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            columns: vec![],
            addr: "127.0.0.1:5995".parse().unwrap(),
            max_connections: 256,
            secret: None,
            key_id: "default".into(),
            cert: None,
            key: None,
            multi: false,
            config: None,
            master_config: None,
            flush_threshold: None,
            durability: None,
            max_partitions: None,
            catalog: false,
            restore_catalog: false,
            http_addr: None,
            http_token: None,
            transport: "tcp".into(),
            xdp_iface: "lo".into(),
            xdp_queue: 0,
            xdp_ring_size: 256,
            grpc_port: 50051,
            no_grpc: false,
            metrics_port: 9100,
            tpm_unseal: false,
        }
    }
}

/// Everything needed to run the server, extracted so tests can bind without blocking.
pub struct BoundServer {
    pub server: Server,
    pub auth: AuthConfig,
    /// Single-table mode: the schema of the default table.
    pub schema: Vec<String>,
    pub tls_enabled: bool,
    /// Single-table mode: the opened engine (None in --multi mode).
    pub engine: Option<Arc<TableEngine>>,
    /// Multi-table mode: the opened registry (None in single-table mode).
    pub registry: Option<Arc<TableRegistry>>,
    /// `--catalog` mode: ShardCatalog at `<dir>/catalog/` (None otherwise). L105.
    pub catalog: Option<Arc<ShardCatalog>>,
    /// Resolved EngineOptions (flush_threshold, durability) after full precedence chain.
    pub engine_opts_flush_threshold: usize,
    /// Master VvConfig loaded from `--master-config` (None when flag not used). L123.
    pub master_config: Option<VvConfig>,
}

type ServerBuilder = Box<dyn FnOnce(ServerConfig) -> Result<Server>>;

/// Build [`CliOverrides`] from the parsed [`ServeArgs`].
pub fn cli_overrides_from_args(args: &ServeArgs) -> CliOverrides {
    let mut o = CliOverrides::new();
    if let Some(v) = args.flush_threshold {
        o = o.with_flush_threshold(v);
    }
    if let Some(ref v) = args.durability {
        o = o.with_durability(v.as_str());
    }
    if let Some(v) = args.max_partitions {
        o = o.with_max_partitions(v);
    }
    o
}

fn build_transport_kind(args: &ServeArgs) -> Result<vectorvolt_net::TransportKind> {
    match args.transport.as_str() {
        "tcp" => Ok(vectorvolt_net::TransportKind::Tcp),
        #[cfg(target_os = "linux")]
        "xdp" => Ok(vectorvolt_net::TransportKind::Xdp {
            iface: args.xdp_iface.clone(),
            queue_id: args.xdp_queue,
            ring_size: args.xdp_ring_size,
        }),
        #[cfg(not(target_os = "linux"))]
        "xdp" => anyhow::bail!("--transport xdp is only supported on Linux"),
        other => anyhow::bail!("unknown transport {:?} — valid values: tcp, xdp", other),
    }
}

pub fn bind(args: &ServeArgs) -> Result<BoundServer> {
    // ── Master config (VV-CFG-011, L123) — lowest precedence layer ───────────
    let master_config: Option<VvConfig> = if let Some(ref p) = args.master_config {
        let cfg = VvConfig::from_file(p)
            .with_context(|| format!("load master config '{}'", p.display()))?;
        Some(cfg)
    } else {
        None
    };

    // Defaults sourced from master config (CLI flags override below).
    let mc_flush_threshold: Option<usize> = master_config
        .as_ref()
        .and_then(|mc| mc.storage.l0_flush_threshold)
        .map(|v| v as usize);
    let mc_max_partitions: Option<usize> = master_config
        .as_ref()
        .and_then(|mc| mc.threads.writer_count)
        .map(|v| v as usize);

    // ── Config — full 4-layer precedence chain (L104: VV-ENG-017) ────────────
    let cli_overrides = cli_overrides_from_args(args);
    let (cfg_opts, _cfg_schema) = load_config_full(
        args.config.as_deref(),
        &SystemProfile::probe(),
        &cli_overrides,
    )
    .with_context(|| "load engine config")?;
    // CLI flush_threshold > per-table config > master config > SystemProfile default
    let resolved_flush_threshold = if args.flush_threshold.is_some() {
        cfg_opts.flush_threshold
    } else {
        mc_flush_threshold.unwrap_or(cfg_opts.flush_threshold)
    };
    // CLI max_partitions > master config threads.writer_count > default 4
    let resolved_max_partitions = args
        .max_partitions
        .unwrap_or_else(|| mc_max_partitions.unwrap_or(4));

    // ── Schema / table setup ──────────────────────────────────────────────────
    let (server_builder, schema, engine, registry) = if args.multi {
        // Multi-table mode: open catalog root.
        std::fs::create_dir_all(&args.dir)
            .with_context(|| format!("create catalog root {}", args.dir.display()))?;
        let opts = cfg_opts;
        let reg = Arc::new(
            TableRegistry::open(&args.dir, opts)
                .with_context(|| format!("open catalog at {}", args.dir.display()))?,
        );
        let table_count = reg.list_tables().len();
        eprintln!(
            "catalog at {}  ({} table(s) restored)",
            args.dir.display(),
            table_count
        );
        let reg2 = Arc::clone(&reg);
        (
            Box::new(move |cfg| Server::bind_registry(reg2, cfg).with_context(|| "bind registry"))
                as ServerBuilder,
            vec![],
            None,
            Some(reg),
        )
    } else {
        // Single-table (legacy) mode.
        let schema_path = args.dir.join(SCHEMA_FILE);
        let schema: Vec<String> = if schema_path.exists() {
            std::fs::read_to_string(&schema_path)
                .with_context(|| format!("read {}", schema_path.display()))?
                .lines()
                .map(str::to_owned)
                .filter(|s| !s.is_empty())
                .collect()
        } else if args.columns.is_empty() {
            bail!(
                "no table found at {} — pass --columns to create one",
                args.dir.display()
            );
        } else {
            std::fs::create_dir_all(&args.dir)
                .with_context(|| format!("create dir {}", args.dir.display()))?;
            std::fs::write(&schema_path, args.columns.join("\n"))
                .with_context(|| "write schema.txt")?;
            eprintln!(
                "created table at {}  schema: {}",
                args.dir.display(),
                args.columns.join(", ")
            );
            args.columns.clone()
        };
        let eng = Arc::new(
            TableEngine::open(schema.clone(), &args.dir, cfg_opts.flush_threshold)
                .with_context(|| "open engine")?,
        );
        let eng2 = Arc::clone(&eng);
        let addr = args.addr;
        (
            Box::new(move |cfg| Server::bind(eng2, cfg).with_context(|| format!("bind {}", addr)))
                as ServerBuilder,
            schema,
            Some(eng),
            None,
        )
    };

    // ── Auth ──────────────────────────────────────────────────────────────────
    let auth = if let Some(hex) = &args.secret {
        let bytes = hex::decode(hex.trim()).with_context(
            || "secret must be a hex string (e.g. produced by `openssl rand -hex 32`)",
        )?;
        let secret = Secret::new(bytes).with_context(|| "invalid secret")?;
        AuthConfig::disabled().with_scoped_key(args.key_id.clone(), secret, Scope::ReadWrite)
    } else {
        AuthConfig::disabled()
    };

    // ── TLS ───────────────────────────────────────────────────────────────────
    let tls_enabled = args.cert.is_some();
    let tls = match (&args.cert, &args.key) {
        (Some(cert), Some(key)) => {
            let tls = ServerTlsConfig::from_files(cert, key)
                .with_context(|| "load TLS certificate / key")?;
            Some(tls)
        }
        (None, None) => None,
        _ => bail!("--cert and --key must both be provided, or neither"),
    };

    // ── Transport ─────────────────────────────────────────────────────────────
    let transport_kind = build_transport_kind(args)?;

    // ── Server ────────────────────────────────────────────────────────────────
    let mut config = ServerConfig::new(args.addr)
        .with_auth(auth.clone())
        .with_transport(transport_kind);
    config.max_connections = args.max_connections;
    if let Some(t) = tls {
        config = config.with_tls(t);
    }

    let server = server_builder(config)?;

    // ── ShardCatalog (L105/L108: VV-CTL-001/004) ────────────────────────────
    // Opened at <dir>/catalog/ when --catalog is set.  Uses with_registry so
    // schema is auto-persisted.  --restore-catalog reloads prior-session tables.
    let catalog = if args.catalog {
        let catalog_dir = args.dir.join("catalog");
        let registry_dir = args.dir.join("registry");
        std::fs::create_dir_all(&catalog_dir)
            .with_context(|| format!("create catalog dir {}", catalog_dir.display()))?;
        let cat = ShardCatalog::with_registry(&catalog_dir, &registry_dir)
            .with_context(|| "open catalog with registry")?;
        if args.restore_catalog {
            cat.restore_all(resolved_flush_threshold, resolved_max_partitions)
                .with_context(|| "restore catalog from registry")?;
        }
        Some(Arc::new(cat))
    } else {
        None
    };

    Ok(BoundServer {
        server,
        auth,
        schema,
        tls_enabled,
        engine,
        registry,
        catalog,
        engine_opts_flush_threshold: resolved_flush_threshold,
        master_config,
    })
}

pub fn run(args: ServeArgs) -> Result<()> {
    // ── TPM unseal (VV-SEC-005) ───────────────────────────────────────────────
    if args.tpm_unseal {
        let sealed_path = args.dir.join("tpm_sealed.bin");
        match vectorvolt_tpm::unseal(&sealed_path) {
            Ok(key_hex) => {
                // Expose as env var for vectorvolt-crypto to pick up.
                unsafe {
                    std::env::set_var(
                        "VECTORVOLT_MASTER_KEY",
                        String::from_utf8_lossy(&key_hex).as_ref(),
                    )
                };
                eprintln!(
                    "TPM unseal: master key loaded from {}",
                    sealed_path.display()
                );
            }
            Err(e) => anyhow::bail!("TPM unseal failed — refusing to start: {e}"),
        }
    }

    let BoundServer {
        server,
        auth,
        schema,
        tls_enabled,
        engine,
        registry,
        catalog,
        engine_opts_flush_threshold,
        master_config,
    } = bind(&args)?;

    let local = server.local_addr()?;
    let stats = server.stats();

    // ── Startup banner ────────────────────────────────────────────────────────
    eprintln!("VectorVolt server ready");
    eprintln!("  address   : {local}");
    if args.multi {
        eprintln!("  catalog   : {}", args.dir.display());
        let names = registry
            .as_ref()
            .map(|r| r.list_tables())
            .unwrap_or_default();
        if names.is_empty() {
            eprintln!("  tables    : (empty — use create-table to add one)");
        } else {
            eprintln!("  tables    : {}", names.join(", "));
        }
    } else {
        eprintln!("  table     : {}", args.dir.display());
        eprintln!("  schema    : {}", schema.join(", "));
    }
    if let Some(ref cat) = catalog {
        eprintln!("  catalog   : {} table(s) ready", cat.table_count());
    }
    eprintln!(
        "  auth      : {}",
        if auth.is_enabled() {
            "HMAC-SHA3 (enabled)"
        } else {
            "disabled"
        }
    );
    eprintln!(
        "  tls       : {}",
        if tls_enabled { "enabled" } else { "plaintext" }
    );
    eprintln!("  max conns : {}", args.max_connections);
    eprintln!("  flush thr : {} rows", engine_opts_flush_threshold);
    eprintln!("  transport : {}", args.transport);
    if let Some(ref mc) = master_config {
        let sections: Vec<&str> = [
            mc.hal != Default::default(),
            mc.threads != Default::default(),
            mc.storage != Default::default(),
            mc.observability != Default::default(),
            mc.network != Default::default(),
            mc.query != Default::default(),
            !mc.tables.is_empty(),
        ]
        .iter()
        .zip([
            "hal",
            "threads",
            "storage",
            "observability",
            "network",
            "query",
            "tables",
        ])
        .filter_map(|(active, name)| if *active { Some(name) } else { None })
        .collect();
        eprintln!("  master cfg: active sections: [{}]", sections.join(", "));
    }

    // ── HTTP gateway (optional) ───────────────────────────────────────────────
    let http_token = args.http_token.clone();
    if let Some(http_addr) = args.http_addr {
        // Build a TableRegistry view for the HTTP layer.
        // In multi-table mode reuse the existing registry; in single-table mode
        // wrap the one engine as "default" so HTTP can address it.
        let http_reg: std::sync::Arc<vectorvolt_engine::TableRegistry> =
            if let Some(ref reg) = registry {
                std::sync::Arc::clone(reg)
            } else if let Some(ref eng) = engine {
                std::sync::Arc::new(vectorvolt_engine::TableRegistry::new_with_default(
                    std::sync::Arc::clone(eng),
                ))
            } else {
                unreachable!("either registry or engine must be present")
            };
        std::thread::Builder::new()
            .name("http-gateway".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    let mut cfg = HttpServerConfig::new(http_addr);
                    if let Some(tok) = http_token {
                        cfg = cfg.with_token(tok);
                    }
                    let cfg = cfg;
                    match HttpServer::bind(http_reg, cfg).await {
                        Ok(srv) => {
                            eprintln!(
                                "  http      : http://{}",
                                srv.local_addr().unwrap_or(http_addr)
                            );
                            if let Err(e) = srv.serve().await {
                                eprintln!("HTTP gateway error: {e}");
                            }
                        }
                        Err(e) => eprintln!("HTTP gateway bind failed: {e}"),
                    }
                });
            })
            .expect("spawn http-gateway thread");
    } else {
        eprintln!("  http      : disabled (use --http-addr to enable)");
    }

    // ── gRPC write server (optional, VV-NET-004..016 / S16) ──────────────────
    let mut grpc_shutdown = if !args.no_grpc
        && args.grpc_port > 0
        && let Some(ref eng) = engine
    {
        let grpc_addr: SocketAddr = format!("127.0.0.1:{}", args.grpc_port).parse().unwrap();
        let grpc_engine = Arc::clone(eng);
        use vectorvolt_server::GrpcServer;
        let grpc_srv = GrpcServer::standalone(grpc_addr, grpc_engine);
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        std::thread::Builder::new()
            .name("grpc-server".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    eprintln!("  grpc      : grpc://{grpc_addr}");
                    if let Err(e) = grpc_srv.run(rx).await {
                        eprintln!("gRPC server error: {e}");
                    }
                });
            })
            .expect("spawn grpc-server thread");
        Some(tx)
    } else {
        eprintln!("  grpc      : disabled");
        None
    };

    // ── Prometheus metrics endpoint (VV-OBS-005) ─────────────────────────────
    if args.metrics_port > 0 {
        use std::sync::atomic::Ordering;
        use vectorvolt_metrics::{MetricsRegistry, MetricsServer};
        let metrics_reg = std::sync::Arc::new(MetricsRegistry::default());
        let metrics_stats = server.stats();
        let sync_reg = std::sync::Arc::clone(&metrics_reg);
        std::thread::Builder::new()
            .name("metrics-sync".into())
            .spawn(move || {
                loop {
                    sync_reg.connections_accepted.store(
                        metrics_stats.connections_accepted.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    sync_reg.requests_total.store(
                        metrics_stats.requests_handled.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    sync_reg.errors_total.store(
                        metrics_stats.protocol_errors.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            })
            .expect("spawn metrics-sync thread");
        let mp = args.metrics_port;
        std::thread::Builder::new()
            .name("metrics-http".into())
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    MetricsServer::new(metrics_reg).serve(mp).await;
                });
            })
            .expect("spawn metrics-http thread");
        eprintln!(
            "  metrics   : http://127.0.0.1:{}/metrics",
            args.metrics_port
        );
    } else {
        eprintln!("  metrics   : disabled (use --metrics-port to enable)");
    }

    eprintln!();
    eprintln!("Ready. Press Ctrl-C to stop.");

    // Register a Ctrl-C handler so we can flush before exit.
    let shutdown_handle = server.shutdown_handle();
    ctrlc::set_handler(move || {
        eprintln!("\nShutting down…");
        // In single-table mode flush the one engine; in multi-table mode each
        // table flushes itself when the Arc is dropped, so we just shutdown.
        if let Some(ref eng) = engine
            && let Ok(Some(r)) = eng.flush()
        {
            eprintln!("  flushed {} buffered row(s)", r.row_count);
        }
        let c = stats
            .connections_accepted
            .load(std::sync::atomic::Ordering::Relaxed);
        let r = stats
            .requests_handled
            .load(std::sync::atomic::Ordering::Relaxed);
        eprintln!("  connections served : {c}");
        eprintln!("  requests handled   : {r}");
        shutdown_handle.shutdown();
        if let Some(gtx) = grpc_shutdown.take() {
            let _ = gtx.send(());
        }
    })
    .with_context(|| "install Ctrl-C handler")?;

    server.serve().with_context(|| "server error")?;
    Ok(())
}
