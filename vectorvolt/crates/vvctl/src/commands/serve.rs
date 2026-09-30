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
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::node_runtime::{ClusterNode, ClusterStatusServer};
use vectorvolt_crypto::{EncryptedReader, EncryptedWriter, KeyStore};
use vectorvolt_engine::{
    CliOverrides, ShardCatalog, SystemProfile, TableEngine, TableRegistry, VvConfig,
    load_config_full,
};
use vectorvolt_http::{HttpServer, HttpServerConfig};
use vectorvolt_net::tls::ServerTlsConfig;
use vectorvolt_net::{AuthConfig, Scope, Secret, Server, ServerConfig};
use vectorvolt_reader::server::{PgConfig, PgServer};

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

    /// Encrypt table metadata at rest with AES-256-GCM (`vectorvolt-crypto`, VV-SEC-001/002).
    ///
    /// Requires a master key: either unseal one from the TPM with `--tpm-unseal`, or
    /// export `VECTORVOLT_MASTER_KEY` (64 hex chars, see `vvctl install --gen-key`)
    /// before starting the server. Fails immediately if neither is present.
    ///
    /// Today this covers the table's `schema.txt` metadata file (written/read via
    /// `vectorvolt_crypto::{EncryptedWriter,EncryptedReader}`). Row data in `.vvsst`
    /// partitions and WAL segments is NOT yet covered — see VV-SEC-006 follow-up;
    /// those readers are mmap-based and require a format change to support
    /// transparent decryption without losing the zero-copy read path.
    #[arg(long)]
    pub encrypt_at_rest: bool,

    /// Also start a PostgreSQL wire-protocol (pgwire) listener on this port.
    ///
    /// Shares the SAME `TableRegistry` as the wire-protocol/HTTP/gRPC listeners
    /// started by this process, so data inserted via any listener is visible to
    /// SQL clients connecting over pgwire (and vice versa). This is the
    /// integrated way to expose VectorVolt to Postgres-wire clients; prefer it
    /// over the standalone `vvctl reader start` command, which opens its own
    /// independent `TableRegistry` against the same data directory and is now a
    /// fallback/dev-only path (see `commands::reader`). Disabled if omitted.
    #[arg(long)]
    pub pgwire_port: Option<u16>,

    /// Start a `vectorvolt-cluster` node bound to this address (gossip/Raft
    /// membership listener) and expose its live status over
    /// `--cluster-status-port` (Gap #3).
    ///
    /// NOTE: `vectorvolt-cluster` does not yet implement wire-level gossip
    /// transport (see `vectorvolt_cluster::node_runtime` docs), so this
    /// starts a real, observable node with real membership/gossip state, but
    /// `--cluster-seeds` peers are recorded as configured-only and are not
    /// yet reachable over the network. Disabled if omitted.
    #[arg(long)]
    pub cluster_bind: Option<SocketAddr>,

    /// Comma-separated seed peer addresses to record for this cluster node
    /// (see `--cluster-bind`). Purely informational until gossip transport
    /// exists.
    #[arg(long, value_delimiter = ',')]
    pub cluster_seeds: Vec<SocketAddr>,

    /// Explicit node id (UUID) for this cluster node. Auto-generated if
    /// omitted.
    #[arg(long)]
    pub cluster_node_id: Option<String>,

    /// Port to serve `GET /cluster/status` JSON on. Only used when
    /// `--cluster-bind` is set. Defaults to 7900.
    #[arg(long, default_value_t = 7900)]
    pub cluster_status_port: u16,
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
            encrypt_at_rest: false,
            pgwire_port: None,
            cluster_bind: None,
            cluster_seeds: vec![],
            cluster_node_id: None,
            cluster_status_port: 7900,
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
    /// Whether `--encrypt-at-rest` is active (schema metadata encrypted, VV-SEC-006).
    pub encrypt_at_rest: bool,
}

/// Resolve the AES-256-GCM master key for `--encrypt-at-rest` from
/// `VECTORVOLT_MASTER_KEY` (set directly, or by `--tpm-unseal` earlier in
/// `run()`). Fails with a clear message if neither is present.
fn resolve_master_key() -> Result<KeyStore> {
    KeyStore::from_env().map_err(|e| {
        anyhow::anyhow!(
            "--encrypt-at-rest requires a master key: {e}. \
             Set VECTORVOLT_MASTER_KEY (see `vvctl install --gen-key`) or pass --tpm-unseal."
        )
    })
}

/// Read `schema.txt`, transparently decrypting it first when `encrypt` is set.
fn read_schema_file(path: &std::path::Path, encrypt: bool) -> Result<Vec<String>> {
    let contents = if encrypt {
        let key = resolve_master_key()?;
        let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mut reader = EncryptedReader::new(file, &key)
            .with_context(|| format!("decrypt {}", path.display()))?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut reader, &mut s)
            .with_context(|| format!("read decrypted {}", path.display()))?;
        s
    } else {
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    };
    Ok(contents
        .lines()
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
        .collect())
}

/// Write `schema.txt`, transparently encrypting it first when `encrypt` is set.
fn write_schema_file(path: &std::path::Path, columns: &[String], encrypt: bool) -> Result<()> {
    let plaintext = columns.join("\n");
    if encrypt {
        let key = resolve_master_key()?;
        let file =
            std::fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
        let mut writer = EncryptedWriter::new(file, key);
        std::io::Write::write_all(&mut writer, plaintext.as_bytes())
            .with_context(|| format!("write {}", path.display()))?;
        std::io::Write::flush(&mut writer)
            .with_context(|| format!("encrypt+flush {}", path.display()))?;
    } else {
        std::fs::write(path, plaintext).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
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
    if args.encrypt_at_rest && args.multi {
        bail!(
            "--encrypt-at-rest is not yet supported with --multi (catalog schemas aren't \
             routed through schema.txt) — see VV-SEC-006 follow-up"
        );
    }
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
            read_schema_file(&schema_path, args.encrypt_at_rest)?
        } else if args.columns.is_empty() {
            bail!(
                "no table found at {} — pass --columns to create one",
                args.dir.display()
            );
        } else {
            std::fs::create_dir_all(&args.dir)
                .with_context(|| format!("create dir {}", args.dir.display()))?;
            write_schema_file(&schema_path, &args.columns, args.encrypt_at_rest)?;
            eprintln!(
                "created table at {}  schema: {}{}",
                args.dir.display(),
                args.columns.join(", "),
                if args.encrypt_at_rest {
                    " (schema.txt encrypted at rest)"
                } else {
                    ""
                }
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
        encrypt_at_rest: args.encrypt_at_rest,
    })
}

/// Resolve any cross-table transaction left dangling by a crash between
/// "PREPARE" and "COMMITTED" in a previous run's intent log, before
/// accepting new connections. Every SQL-layer listener (HTTP/pgwire) opens
/// its own `SqlExecutor` pointed at the same registry `base_dir`, so the
/// intent log path is deterministic here even though `run()` hasn't
/// constructed a `SqlExecutor` itself. Factored out from `run()` so it's
/// directly testable against a real registry without spinning up the full
/// server (threads, sockets, Ctrl-C handler).
pub fn run_startup_recovery(registry: &Arc<vectorvolt_engine::TableRegistry>) {
    let intent_log_path = registry.base_dir().join("_txn_intent.log");
    if !intent_log_path.exists() {
        return;
    }
    match vectorvolt_engine::recover_pending_transactions(&intent_log_path, |table| {
        registry.get(table).ok()
    }) {
        Ok(actions) if !actions.is_empty() => {
            eprintln!(
                "Recovered {} dangling cross-table transaction(s) from a previous crash:",
                actions.len()
            );
            for action in &actions {
                eprintln!(
                    "  {} -> {:?} (committed: {:?}, failed: {:?})",
                    action.txn_uuid,
                    action.resolved_state,
                    action.tables_committed,
                    action.tables_failed
                );
            }
        }
        Ok(_) => {}
        Err(e) => eprintln!("WARNING: transaction recovery failed: {e}"),
    }
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
        encrypt_at_rest,
    } = bind(&args)?;

    let local = server.local_addr()?;
    let stats = server.stats();

    // ── Auth safety check (VV-SEC-006) ───────────────────────────────────────
    // `--secret` is the real gate here (HMAC-SHA3 challenge-response, enforced in
    // vectorvolt-net's Server) — vectorvolt-pam is a separate, OS-level SSH PAM
    // module and is not something the wire-protocol server calls into. Warn loudly
    // if auth is off and the bind address is not loopback-only.
    if !auth.is_enabled() && !local.ip().is_loopback() {
        eprintln!(
            "WARNING: auth is disabled and the server is bound to a non-loopback \
             address ({local}) — any client that can reach this port can read and \
             write data. Pass --secret <hex> to require authentication."
        );
    }

    // ── MVCC Phase 3 crash recovery ──────────────────────────────────────────
    let recovery_registry: Arc<vectorvolt_engine::TableRegistry> = if let Some(ref reg) = registry {
        Arc::clone(reg)
    } else if let Some(ref eng) = engine {
        Arc::new(vectorvolt_engine::TableRegistry::new_with_default(
            Arc::clone(eng),
        ))
    } else {
        unreachable!("either registry or engine must be present")
    };
    run_startup_recovery(&recovery_registry);

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
    eprintln!(
        "  encrypt   : {}",
        if encrypt_at_rest {
            "schema.txt (AES-256-GCM)"
        } else {
            "disabled (use --encrypt-at-rest)"
        }
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
    // Gap #6: graceful-shutdown signal for the HTTP gateway, fired from the
    // Ctrl-C handler below so in-flight requests finish instead of being cut
    // off by an abrupt process exit.
    let mut http_shutdown: Option<tokio::sync::oneshot::Sender<()>> = None;
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
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        http_shutdown = Some(tx);
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
                            let shutdown = async move {
                                let _ = rx.await;
                            };
                            if let Err(e) = srv.serve_with_shutdown(shutdown).await {
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

    // ── pgwire listener (optional) ───────────────────────────────────────────
    // Shares the same TableRegistry as the wire-protocol/HTTP/gRPC listeners
    // above (single-table mode wraps the one engine as "default", same as the
    // HTTP gateway does) so SQL clients over pgwire see the same data as every
    // other listener in this process — this is what makes `vvctl serve` the
    // one integrated server for Postgres-wire clients (Gap #2 fix).
    // Gap #6: graceful-shutdown signal for the pgwire listener (stop accepting
    // new connections on Ctrl-C instead of the socket vanishing mid-handshake).
    let mut pgwire_shutdown: Option<tokio::sync::oneshot::Sender<()>> = None;
    if let Some(pgwire_port) = args.pgwire_port {
        let pg_reg: Arc<vectorvolt_engine::TableRegistry> = if let Some(ref reg) = registry {
            Arc::clone(reg)
        } else if let Some(ref eng) = engine {
            Arc::new(vectorvolt_engine::TableRegistry::new_with_default(
                Arc::clone(eng),
            ))
        } else {
            unreachable!("either registry or engine must be present")
        };
        let pgwire_addr: SocketAddr = format!("127.0.0.1:{pgwire_port}").parse().unwrap();
        let pg_dir = args.dir.clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        pgwire_shutdown = Some(tx);
        std::thread::Builder::new()
            .name("pgwire-server".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    let config = PgConfig::new(pgwire_addr, pg_dir);
                    let server = PgServer::with_registry(config, pg_reg);
                    eprintln!("  pgwire    : postgres://{pgwire_addr}");
                    let shutdown = async move {
                        let _ = rx.await;
                    };
                    if let Err(e) = server.serve_until(shutdown).await {
                        eprintln!("pgwire server error: {e}");
                    }
                });
            })
            .expect("spawn pgwire-server thread");
    } else {
        eprintln!("  pgwire    : disabled (use --pgwire-port to enable)");
    }

    // ── Cluster node (optional, Gap #3) ──────────────────────────────────────
    // Starts a real vectorvolt-cluster node (NodeId + MembershipList +
    // GossipEngine) alongside this single-node server so its state is
    // observable via GET /cluster/status. See
    // vectorvolt_cluster::node_runtime for what is and is not real here:
    // membership/gossip data structures are the genuine ones exercised by
    // vectorvolt-cluster's own tests, but there is no wire-level gossip
    // transport yet, so --cluster-seeds peers are recorded as
    // configured-only, not claimed as discovered/reachable.
    if let Some(cluster_bind) = args.cluster_bind {
        let node_id = match args.cluster_node_id.as_deref() {
            Some(s) => match uuid::Uuid::parse_str(s) {
                Ok(u) => Some(NodeId(u)),
                Err(e) => {
                    eprintln!("invalid --cluster-node-id {s:?}: {e}");
                    None
                }
            },
            None => None,
        };
        let cluster_node = ClusterNode::start(cluster_bind, args.cluster_seeds.clone(), node_id);
        eprintln!(
            "  cluster   : node {} on {} ({} configured seed(s))",
            cluster_node.status().node_id,
            cluster_bind,
            args.cluster_seeds.len()
        );
        let status_port = args.cluster_status_port;
        std::thread::Builder::new()
            .name("cluster-status".into())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    eprintln!("  cluster   : status http://127.0.0.1:{status_port}/cluster/status");
                    ClusterStatusServer::new(cluster_node)
                        .serve(status_port)
                        .await;
                });
            })
            .expect("spawn cluster-status thread");
    } else {
        eprintln!("  cluster   : disabled (use --cluster-bind to enable)");
    }

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
        // Gap #6: signal the HTTP gateway and pgwire listener too, so they stop
        // accepting new work (HTTP additionally drains in-flight requests via
        // axum's graceful shutdown) instead of just vanishing when main() exits.
        if let Some(htx) = http_shutdown.take() {
            let _ = htx.send(());
        }
        if let Some(ptx) = pgwire_shutdown.take() {
            let _ = ptx.send(());
        }
    })
    .with_context(|| "install Ctrl-C handler")?;

    server.serve().with_context(|| "server error")?;
    Ok(())
}
