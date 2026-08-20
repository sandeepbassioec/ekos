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
    CliOverrides, ShardCatalog, SystemProfile, TableEngine, TableRegistry, load_config_full,
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

pub fn bind(args: &ServeArgs) -> Result<BoundServer> {
    // ── Config — full 4-layer precedence chain (L104: VV-ENG-017) ────────────
    let cli_overrides = cli_overrides_from_args(args);
    let (cfg_opts, _cfg_schema) = load_config_full(
        args.config.as_deref(),
        &SystemProfile::probe(),
        &cli_overrides,
    )
    .with_context(|| "load engine config")?;
    let resolved_flush_threshold = cfg_opts.flush_threshold;

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

    // ── Server ────────────────────────────────────────────────────────────────
    let mut config = ServerConfig::new(args.addr).with_auth(auth.clone());
    config.max_connections = args.max_connections;
    if let Some(t) = tls {
        config = config.with_tls(t);
    }

    let server = server_builder(config)?;

    // ── ShardCatalog (L105: VV-CTL-001) ─────────────────────────────────────
    // Opened at <dir>/catalog/ when --catalog is set.  Stored in BoundServer
    // for direct TxnManager access; S4 will add native catalog wire binding.
    let catalog = if args.catalog {
        let catalog_dir = args.dir.join("catalog");
        std::fs::create_dir_all(&catalog_dir)
            .with_context(|| format!("create catalog dir {}", catalog_dir.display()))?;
        Some(Arc::new(ShardCatalog::new(&catalog_dir)))
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
    })
}

pub fn run(args: ServeArgs) -> Result<()> {
    let BoundServer {
        server,
        auth,
        schema,
        tls_enabled,
        engine,
        registry,
        catalog,
        engine_opts_flush_threshold,
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
    })
    .with_context(|| "install Ctrl-C handler")?;

    server.serve().with_context(|| "server error")?;
    Ok(())
}
