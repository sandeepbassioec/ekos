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
use vectorvolt_engine::TableEngine;
use vectorvolt_net::tls::ServerTlsConfig;
use vectorvolt_net::{AuthConfig, Scope, Secret, Server, ServerConfig};

const SCHEMA_FILE: &str = "schema.txt";
const FLUSH_THRESHOLD: usize = 1_048_576;

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
}

/// Everything needed to run the server, extracted so tests can bind without blocking.
pub struct BoundServer {
    pub server: Server,
    pub auth: AuthConfig,
    pub schema: Vec<String>,
    pub tls_enabled: bool,
    pub engine: Arc<TableEngine>,
}

pub fn bind(args: &ServeArgs) -> Result<BoundServer> {
    // ── Schema / table setup ──────────────────────────────────────────────────
    let schema_path = args.dir.join(SCHEMA_FILE);

    let schema: Vec<String> = if schema_path.exists() {
        std::fs::read_to_string(&schema_path)
            .with_context(|| format!("read {}", schema_path.display()))?
            .lines()
            .map(str::to_owned)
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        if args.columns.is_empty() {
            bail!(
                "no table found at {} — pass --columns to create one",
                args.dir.display()
            );
        }
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

    let engine = Arc::new(
        TableEngine::open(schema.clone(), &args.dir, FLUSH_THRESHOLD)
            .with_context(|| "open engine")?,
    );

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

    let server =
        Server::bind(Arc::clone(&engine), config).with_context(|| format!("bind {}", args.addr))?;

    Ok(BoundServer {
        server,
        auth,
        schema,
        tls_enabled,
        engine,
    })
}

pub fn run(args: ServeArgs) -> Result<()> {
    let BoundServer {
        server,
        auth,
        schema,
        tls_enabled,
        engine,
    } = bind(&args)?;

    let local = server.local_addr()?;
    let stats = server.stats();

    // ── Startup banner ────────────────────────────────────────────────────────
    eprintln!("VectorVolt server ready");
    eprintln!("  address   : {local}");
    eprintln!("  table     : {}", args.dir.display());
    eprintln!("  schema    : {}", schema.join(", "));
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
    eprintln!("  flush thr : {} rows", FLUSH_THRESHOLD);
    eprintln!();
    eprintln!("Ready. Press Ctrl-C to stop.");

    // Register a Ctrl-C handler so we can flush before exit.
    let shutdown_handle = server.shutdown_handle();
    let engine_for_signal = Arc::clone(&engine);
    ctrlc::set_handler(move || {
        eprintln!("\nShutting down…");
        if let Ok(Some(r)) = engine_for_signal.flush() {
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
