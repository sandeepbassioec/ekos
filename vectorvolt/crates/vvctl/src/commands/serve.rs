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
//! # with TLS on every listener (wire, HTTP, gRPC, pgwire)
//! vvctl serve ./data --tls-cert server.pem --tls-key server.key
//!
//! # mutual TLS: clients must present a certificate issued by ca.pem
//! vvctl serve ./data --tls-cert server.pem --tls-key server.key --tls-client-ca ca.pem
//! ```

#[path = "serve_sinks.rs"]
pub mod sinks;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::Args;
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::node_runtime::{ClusterNode, ClusterStatusServer};
use vectorvolt_crypto::{EncryptedReader, EncryptedWriter, KeyStore};
use vectorvolt_engine::{
    CliOverrides, EngineOptions, ShardCatalog, SystemProfile, TableEngine, TableRegistry, VvConfig,
    load_config_full,
};
use vectorvolt_http::{HttpServer, HttpServerConfig};
use vectorvolt_net::tls::ServerTlsConfig;
use vectorvolt_net::{AuthConfig, Scope, Secret, Server, ServerConfig};
use vectorvolt_reader::auth::AuthMethod;
use vectorvolt_reader::server::{PgConfig, PgServer};
use vectorvolt_replication::{
    ReplicationClient, ReplicationConfig, ReplicationMode, ReplicationRole, ReplicationServer,
    RoleCell,
};

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

    /// Scope of the `--secret` key. Defaults to `admin` with `--multi` (which
    /// needs it for table selection and DDL) and `read-write` otherwise.
    #[arg(long, value_enum)]
    pub key_scope: Option<KeyScope>,

    /// File of additional keys, one per line: `<key_id> <scope> <hex-secret>`,
    /// where scope is `read-only`, `read-write` or `admin`. Blank lines and
    /// lines starting with `#` are ignored.
    #[arg(long)]
    pub key_file: Option<PathBuf>,

    /// PEM certificate (chain) for TLS. Applies to every listener this
    /// process starts: the wire protocol, `--http-addr`, gRPC and
    /// `--pgwire-port` (pgwire upgrades on SSLRequest). `--cert` is accepted
    /// as an alias.
    #[arg(long = "tls-cert", alias = "cert", requires = "key")]
    pub cert: Option<PathBuf>,

    /// PEM private key for `--tls-cert`. `--key` is accepted as an alias.
    #[arg(long = "tls-key", alias = "key", requires = "cert")]
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

    /// Declare downstream sinks (kafka / postgres / redis_stream) in a YAML or
    /// TOML file. Every sink must connect at startup or the server refuses to
    /// start; committed mutations are then tailed from each table's WAL and
    /// delivered to all sinks (at-least-once) via a durable outbox under
    /// `<dir>/events`. Works with `--encrypt-at-rest` (frames are decrypted
    /// with the same key the engine uses).
    #[arg(long)]
    pub sinks_config: Option<PathBuf>,

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

    /// Interface the gRPC server binds (default loopback only).
    #[arg(long, default_value = "127.0.0.1")]
    pub grpc_host: std::net::IpAddr,

    /// Bearer token every gRPC call must present (`authorization: Bearer …`).
    ///
    /// Required whenever `--secret` protects the wire listener — otherwise gRPC
    /// would be an unauthenticated way around it — and whenever `--grpc-host`
    /// is not a loopback address. Without it in those cases gRPC stays off.
    /// Falls back to the `VECTORVOLT_GRPC_TOKEN` environment variable.
    #[arg(long)]
    pub grpc_token: Option<String>,

    /// Expose a Prometheus `/metrics` endpoint on this port (default 9100, 0 = disabled).
    ///
    /// Serves connections-accepted, active-connections, requests-total, errors-total,
    /// WAL entries, and byte counters in Prometheus text exposition format.
    #[arg(long, default_value = "9100")]
    pub metrics_port: u16,

    /// Interface the metrics endpoint binds (default loopback only). Use
    /// `0.0.0.0` to let a remote Prometheus scrape it.
    #[arg(long, default_value = "127.0.0.1")]
    pub metrics_host: std::net::IpAddr,

    /// Unseal the master key from the TPM blob at `<dir>/tpm_sealed.bin` on startup.
    ///
    /// Fails immediately if the TPM is not present, the blob is missing, or the PCR
    /// policy does not match (boot chain tampered). Requires Linux + TPM 2.0 hardware.
    #[arg(long)]
    pub tpm_unseal: bool,

    /// Encrypt data at rest with AES-256-GCM (`vectorvolt-crypto`, VV-SEC-001/002).
    ///
    /// Requires a master key: either unseal one from the TPM with `--tpm-unseal`, or
    /// export `VECTORVOLT_MASTER_KEY` (64 hex chars, see `vvctl install --gen-key`)
    /// before starting the server. Fails immediately if neither is present.
    ///
    /// Covers every table's WAL payloads (single-table and `--multi`, PP-51), the
    /// single-table `schema.txt`, and enables SQL column encryption (`INSERT
    /// ENCRYPTED`) on the pgwire listener under a key derived from the master key.
    /// `.vvsst` partitions, the `--multi` `schema.txt` files and the catalog WAL are
    /// NOT yet covered (PP-53/PP-54); the partition readers are mmap-based and need
    /// a format change to decrypt without losing the zero-copy read path.
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

    /// Start a `vectorvolt-cluster` node bound to this address (UDP SWIM
    /// gossip) and expose its live membership and control endpoint on
    /// `--cluster-status-port`. Peers are discovered by joining through
    /// `--cluster-seeds`. Positional argument after the table directory in
    /// `serve <dir> <cluster_bind>`; disabled if omitted.
    pub cluster_bind: Option<SocketAddr>,

    /// Comma-separated seed peer addresses (`host:port` of other nodes'
    /// the cluster-bind address). The node joins the cluster through the first seed
    /// that answers, retrying with backoff in the background.
    #[arg(long, value_delimiter = ',')]
    pub cluster_seeds: Vec<SocketAddr>,

    /// Explicit node id (UUID) for this cluster node. Defaults to the node id
    /// in `<dir>/cluster_identity.json` (written by `vvctl cluster init`),
    /// else auto-generated.
    #[arg(long)]
    pub cluster_node_id: Option<String>,

    /// Port to serve `GET /cluster/status` JSON on. Only used when
    /// a cluster-bind address is given. Defaults to 7900.
    #[arg(long, default_value_t = 7900)]
    pub cluster_status_port: u16,

    /// Listener TLS / pgwire options (D1, D3, D4).
    #[command(flatten)]
    pub listener: ListenerArgs,

    /// WAL-shipping replication options (F5).
    #[command(flatten)]
    pub replication: ReplicationArgs,
}

/// WAL-shipping replication flags (`vectorvolt-replication`).
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ReplicationArgs {
    /// Replication role: `leader` streams committed WAL frames to followers
    /// (needs `--replication-listen`); `follower` pulls them from
    /// `--replicate-from` and is read-only. Single-table mode only.
    #[arg(long)]
    pub replication_role: Option<ReplicationRole>,

    /// Leader address to replicate from (`host:port`). Follower only.
    #[arg(long)]
    pub replicate_from: Option<String>,

    /// Address the leader accepts follower connections on. Leader only.
    #[arg(long)]
    pub replication_listen: Option<SocketAddr>,

    /// When a write counts as replicated: `async` (never wait), `semi-sync`
    /// (one follower applied) or `sync` (replication-factor - 1 followers
    /// durably applied, else the write errors after the ack timeout).
    #[arg(long, default_value = "async")]
    pub replication_mode: ReplicationMode,

    /// Total copies of the data including the leader.
    #[arg(long, default_value_t = 2)]
    pub replication_factor: usize,

    /// Run the epoch-fenced failover control plane (F6): a Raft group across
    /// the nodes elects the write primary, bumps the fencing epoch on every
    /// promotion and re-points followers. Needs `--raft-listen`,
    /// `--replication-listen` and one `--failover-peer` per other node;
    /// replaces `--replication-role`. Single-table mode only.
    #[arg(long)]
    pub failover: bool,

    /// Raft RPC listener (TCP) for `--failover`.
    #[arg(long)]
    pub raft_listen: Option<SocketAddr>,

    /// Another control-plane member: `<uuid>=<raft_addr>,<repl_addr>,<client_addr>`.
    /// Repeatable.
    #[arg(long = "failover-peer")]
    pub failover_peers: Vec<String>,

    /// Client address this node advertises as the primary in "not primary"
    /// errors (defaults to the wire-protocol listen address).
    #[arg(long)]
    pub failover_client_addr: Option<String>,

    /// Join an already running failover group instead of bootstrapping one:
    /// this node asks the `--failover-peer` members (via their replication
    /// addresses) to add it through Raft membership. `--failover-peer` is the
    /// bootstrap list only; membership changes at runtime thereafter.
    #[arg(long)]
    pub failover_join: bool,

    /// Accept sync / semi-sync writes with fewer in-sync replicas than the
    /// configured factor instead of rejecting them with "insufficient
    /// replicas". The degraded state is reported (metric + log) and full
    /// acknowledgement resumes by itself when followers catch up.
    #[arg(long)]
    pub replication_degraded_ok: bool,
}

impl Default for ReplicationArgs {
    fn default() -> Self {
        Self {
            replication_role: None,
            replicate_from: None,
            replication_listen: None,
            replication_mode: ReplicationMode::Async,
            replication_factor: 2,
            failover: false,
            raft_listen: None,
            failover_peers: Vec::new(),
            failover_client_addr: None,
            failover_join: false,
            replication_degraded_ok: false,
        }
    }
}

/// Shared listener options: mutual TLS for every listener, and pgwire bind /
/// TLS / connection-limit settings.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ListenerArgs {
    /// PEM CA bundle for mutual TLS. When set, every TLS listener (wire,
    /// HTTP, gRPC, pgwire) requires clients to present a certificate issued
    /// by one of these CAs. Requires `--tls-cert` / `--tls-key`.
    #[arg(long, requires = "cert")]
    pub tls_client_ca: Option<PathBuf>,

    /// Interface the pgwire listener binds (default loopback only).
    #[arg(long, default_value = "127.0.0.1")]
    pub pgwire_host: IpAddr,

    /// Refuse pgwire clients that do not negotiate TLS (SSLRequest) with
    /// SQLSTATE 28000. Requires `--tls-cert` / `--tls-key`.
    #[arg(long, requires = "cert")]
    pub pgwire_require_tls: bool,

    /// Cap concurrent pgwire sessions; beyond it clients get SQLSTATE 53300.
    /// The SQL-level `SET MAX_CONNECTIONS` (default 100) also applies — the
    /// tighter limit wins.
    #[arg(long)]
    pub pgwire_max_connections: Option<usize>,

    /// How pgwire clients prove they know `--secret`: `scram`
    /// (SCRAM-SHA-256, default), `md5`, or `password` (cleartext — only
    /// sensible with TLS).
    #[arg(long, default_value = "scram", value_parser = parse_auth_method)]
    pub pgwire_auth: AuthMethod,
}

fn parse_auth_method(s: &str) -> std::result::Result<AuthMethod, String> {
    s.parse()
}

impl Default for ListenerArgs {
    fn default() -> Self {
        Self {
            tls_client_ca: None,
            pgwire_host: IpAddr::from([127, 0, 0, 1]),
            pgwire_require_tls: false,
            pgwire_max_connections: None,
            pgwire_auth: AuthMethod::default(),
        }
    }
}

/// TLS material loaded once from `--tls-cert` / `--tls-key` /
/// `--tls-client-ca` and shared by every listener.
#[derive(Clone)]
pub struct ServeTls {
    /// The wire listener's config; its rustls config also drives HTTP and pgwire.
    pub net: ServerTlsConfig,
    /// Raw PEM, for gRPC (tonic builds its own rustls config from PEM).
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
    pub client_ca_pem: Option<Vec<u8>>,
}

impl ServeTls {
    /// True if clients must present a certificate.
    pub fn is_mutual(&self) -> bool {
        self.client_ca_pem.is_some()
    }

    fn describe(&self) -> &'static str {
        if self.is_mutual() {
            "mutual TLS (client certificate required)"
        } else {
            "TLS"
        }
    }
}

/// Load the TLS material named by `args` (`None` when TLS is not configured).
pub fn load_tls(args: &ServeArgs) -> Result<Option<ServeTls>> {
    match (&args.cert, &args.key) {
        (Some(cert), Some(key)) => {
            let cert_pem = std::fs::read(cert)
                .with_context(|| format!("read TLS certificate {}", cert.display()))?;
            let key_pem =
                std::fs::read(key).with_context(|| format!("read TLS key {}", key.display()))?;
            let client_ca_pem = args
                .listener
                .tls_client_ca
                .as_ref()
                .map(|p| {
                    std::fs::read(p).with_context(|| format!("read TLS client CA {}", p.display()))
                })
                .transpose()?;
            let net = match &client_ca_pem {
                Some(ca) => ServerTlsConfig::from_pem_with_client_ca(&cert_pem, &key_pem, ca),
                None => ServerTlsConfig::from_pem(&cert_pem, &key_pem),
            }
            .with_context(|| "load TLS certificate / key")?;
            Ok(Some(ServeTls {
                net,
                cert_pem,
                key_pem,
                client_ca_pem,
            }))
        }
        (None, None) => {
            if args.listener.tls_client_ca.is_some() {
                bail!("--tls-client-ca requires --tls-cert and --tls-key");
            }
            if args.listener.pgwire_require_tls {
                bail!("--pgwire-require-tls requires --tls-cert and --tls-key");
            }
            Ok(None)
        }
        _ => bail!("--tls-cert and --tls-key must both be provided, or neither"),
    }
}

/// The pgwire bind address from `--pgwire-host` and `--pgwire-port`.
pub fn pgwire_addr(args: &ServeArgs, port: u16) -> SocketAddr {
    SocketAddr::new(args.listener.pgwire_host, port)
}

/// Warn when a listener is reachable beyond loopback without TLS: its traffic
/// (credentials included) crosses the network in plaintext. A warning, not
/// an error, so existing deployments keep starting.
pub fn plaintext_exposure_warning(listener: &str, addr: SocketAddr, tls: bool) -> Option<String> {
    if tls || addr.ip().is_loopback() {
        return None;
    }
    Some(format!(
        "WARNING: the {listener} listener is bound to a non-loopback address ({addr}) without \
         TLS — traffic, including credentials, is sent in plaintext. Pass --tls-cert and \
         --tls-key to encrypt it."
    ))
}

fn warn_plaintext(listener: &str, addr: SocketAddr, tls: bool) {
    if let Some(w) = plaintext_exposure_warning(listener, addr, tls) {
        eprintln!("{w}");
    }
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
            key_scope: None,
            key_file: None,
            cert: None,
            key: None,
            multi: false,
            config: None,
            master_config: None,
            sinks_config: None,
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
            grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
            grpc_token: None,
            metrics_port: 9100,
            metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
            tpm_unseal: false,
            encrypt_at_rest: false,
            pgwire_port: None,
            cluster_bind: None,
            cluster_seeds: vec![],
            cluster_node_id: None,
            cluster_status_port: 7900,
            listener: ListenerArgs::default(),
            replication: ReplicationArgs::default(),
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
    /// The TLS material every listener uses (`None` = plaintext).
    pub tls: Option<ServeTls>,
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
    /// The master key when `--encrypt-at-rest` is active (`None` otherwise).
    pub master_key: Option<Arc<KeyStore>>,
    /// F6: write fence every listener consults (allows all until failover starts).
    pub write_gate: super::serve_failover::LateGate,
    /// F6: post-write replication wait every listener runs.
    pub after_write: super::serve_failover::LateGate,
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

/// Domain-separation label for the SQL column-encryption key (A2).
const COLUMN_KEY_LABEL: &[u8] = b"vectorvolt-column-encryption-v1";

/// Derive the SQL column-encryption key (`INSERT ENCRYPTED` / `SELECT DECRYPT`)
/// from the at-rest master key: `SHA3-256(label || master_key)`. Domain
/// separation keeps column ciphertexts and WAL ciphertexts under different keys.
pub fn derive_column_key(master: &KeyStore) -> Result<KeyStore> {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    hasher.update(COLUMN_KEY_LABEL);
    hasher.update(master.as_bytes());
    // `from_bytes` moves the derived key into protected (locked) memory.
    let digest: [u8; 32] = hasher.finalize().into();
    KeyStore::from_bytes(digest)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| "derive column-encryption key")
}

/// Build the pgwire server `vvctl serve --pgwire-port` runs: shares `registry`,
/// is gated by `password` (checked with `auth`: SCRAM-SHA-256, MD5 or
/// cleartext), and has SQL column encryption enabled when
/// `column_key` is set (A2: otherwise `INSERT ENCRYPTED` fails with "column
/// encryption key not configured").
pub fn build_pg_server(
    config: PgConfig,
    registry: Arc<TableRegistry>,
    password: Option<String>,
    auth: AuthMethod,
    column_key: Option<KeyStore>,
) -> PgServer {
    let server = PgServer::with_registry(config, registry)
        .with_auth_method(auth)
        .with_password(password);
    if let Some(key) = column_key {
        server.executor().set_column_encryption_key(key);
    }
    server
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

/// Scope of a wire-protocol key, as given on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum KeyScope {
    ReadOnly,
    ReadWrite,
    Admin,
}

impl From<KeyScope> for Scope {
    fn from(k: KeyScope) -> Self {
        match k {
            KeyScope::ReadOnly => Scope::ReadOnly,
            KeyScope::ReadWrite => Scope::ReadWrite,
            KeyScope::Admin => Scope::Admin,
        }
    }
}

fn parse_secret(hex: &str) -> Result<Secret> {
    let bytes = hex::decode(hex.trim())
        .with_context(|| "secret must be a hex string (e.g. produced by `openssl rand -hex 32`)")?;
    Secret::new(bytes).with_context(|| "invalid secret")
}

/// Wire-protocol credentials from `--secret`/`--key-scope` and `--key-file`.
fn build_auth(args: &ServeArgs) -> Result<AuthConfig> {
    let mut auth = AuthConfig::disabled();
    if let Some(hex) = &args.secret {
        let scope = match args.key_scope {
            Some(s) => s.into(),
            // `--multi` needs Admin for UseTable/DDL (PP-30); single-table
            // serving keeps the historical read-write key.
            None if args.multi => Scope::Admin,
            None => Scope::ReadWrite,
        };
        auth = auth.with_scoped_key(args.key_id.clone(), parse_secret(hex)?, scope);
    }
    if let Some(path) = &args.key_file {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading key file {}", path.display()))?;
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            let [key_id, scope, hex] = parts.as_slice() else {
                anyhow::bail!(
                    "{}:{}: expected `<key_id> <scope> <hex-secret>`",
                    path.display(),
                    n + 1
                );
            };
            let scope = <KeyScope as clap::ValueEnum>::from_str(scope, true).map_err(|_| {
                anyhow::anyhow!(
                    "{}:{}: unknown scope {scope:?} (read-only, read-write, admin)",
                    path.display(),
                    n + 1
                )
            })?;
            let secret =
                parse_secret(hex).with_context(|| format!("{}:{}", path.display(), n + 1))?;
            auth = auth.with_scoped_key(*key_id, secret, scope.into());
        }
    }
    Ok(auth)
}

pub fn bind(args: &ServeArgs) -> Result<BoundServer> {
    // ── Master key (--encrypt-at-rest) — resolved first so it fails fast ─────
    let master_key: Option<Arc<KeyStore>> = if args.encrypt_at_rest {
        Some(Arc::new(resolve_master_key()?))
    } else {
        None
    };
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
        // PP-51: every table in the catalog gets WAL encryption at rest.
        let reg = match &master_key {
            Some(key) => TableRegistry::open_with_encryption(&args.dir, opts, Arc::clone(key)),
            None => TableRegistry::open(&args.dir, opts),
        };
        let reg = Arc::new(reg.with_context(|| format!("open catalog at {}", args.dir.display()))?);
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
        let eng = match &master_key {
            Some(key) => TableEngine::open_with_encryption(
                schema.clone(),
                &args.dir,
                EngineOptions::new(cfg_opts.flush_threshold),
                Arc::clone(key),
            ),
            None => TableEngine::open(schema.clone(), &args.dir, cfg_opts.flush_threshold),
        };
        let eng = Arc::new(eng.with_context(|| "open engine")?);
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
    let auth = build_auth(args)?;

    // ── TLS (shared by every listener, D1) ────────────────────────────────────
    let serve_tls = load_tls(args)?;
    let tls_enabled = serve_tls.is_some();

    // ── Transport ─────────────────────────────────────────────────────────────
    let transport_kind = build_transport_kind(args)?;

    // ── Server ────────────────────────────────────────────────────────────────
    let mut config = ServerConfig::new(args.addr)
        .with_auth(auth.clone())
        .with_transport(transport_kind);
    config.max_connections = args.max_connections;
    if let Some(t) = &serve_tls {
        config = config.with_tls(t.net.clone());
    }
    let write_gate = super::serve_failover::LateGate::default();
    let after_write = super::serve_failover::LateGate::default();
    {
        let g = write_gate.as_gate();
        let a = after_write.as_gate();
        config = config
            .with_write_gate(vectorvolt_net::WriteGate::new(move || g()))
            .with_after_write(vectorvolt_net::WriteGate::new(move || a()));
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
        tls: serve_tls,
        engine,
        registry,
        catalog,
        engine_opts_flush_threshold: resolved_flush_threshold,
        master_config,
        encrypt_at_rest: args.encrypt_at_rest,
        master_key,
        write_gate,
        after_write,
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
        tls,
        engine,
        registry,
        catalog,
        engine_opts_flush_threshold,
        master_config,
        encrypt_at_rest,
        master_key,
        write_gate,
        after_write,
    } = bind(&args)?;

    // ── Sinks (CDC fan-out) ───────────────────────────────────────────────────
    // Connected before any listener starts serving so a bad sink endpoint
    // fails startup loudly instead of silently dropping events.
    let sink_registry = if let Some(ref path) = args.sinks_config {
        let file = sinks::load_sinks(path)?;
        let tables: Box<dyn Fn() -> Vec<sinks::TableWal> + Send> = if let Some(ref reg) = registry {
            let reg = Arc::clone(reg);
            let base = args.dir.clone();
            Box::new(move || {
                reg.list_tables()
                    .into_iter()
                    .map(|t| {
                        let wal = base.join(&t).join("wal");
                        (t, wal)
                    })
                    .collect()
            })
        } else {
            let wal = args.dir.join("wal");
            Box::new(move || vec![("default".to_string(), wal.clone())])
        };
        Some(sinks::start(&file, tables, &args.dir, master_key.clone())?)
    } else {
        None
    };

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
    warn_plaintext("wire-protocol", local, tls_enabled);

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

    // ── Epoch-fenced failover control plane (F6) ─────────────────────────────
    // Started before any listener so their write fences are live from the first
    // request: a node is read-only until Raft elects it primary.
    let _failover = start_failover(
        &args,
        engine.as_ref(),
        master_key.as_ref(),
        local,
        &write_gate,
        &after_write,
    )?;
    let (gate_w, gate_a) = (write_gate.as_gate(), after_write.as_gate());

    // ── Versionstamps, shard plane and sagas on the real engine (F7/F8) ─────
    super::serve_cluster_ext::start(&args, engine.as_ref())?;

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
        tls.as_ref().map_or("plaintext", ServeTls::describe)
    );
    eprintln!(
        "  encrypt   : {}",
        if encrypt_at_rest {
            if args.multi {
                "table WALs + SQL column key (AES-256-GCM)"
            } else {
                "WAL + schema.txt + SQL column key (AES-256-GCM)"
            }
        } else {
            "disabled (use --encrypt-at-rest)"
        }
    );
    if let Some(ref r) = sink_registry {
        eprintln!("  sinks     : {}", r.sink_names().join(", "));
    }
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
        let (http_gate_w, http_gate_a) = (gate_w.clone(), gate_a.clone());
        let http_tls = tls.as_ref().map(|t| t.net.rustls_config());
        warn_plaintext("HTTP", http_addr, http_tls.is_some());
        std::thread::Builder::new()
            .name("http-gateway".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    let mut cfg = HttpServerConfig::new(http_addr);
                    cfg = cfg
                        .with_write_gate(http_gate_w)
                        .with_after_write(http_gate_a);
                    if let Some(tok) = http_token {
                        cfg = cfg.with_token(tok);
                    }
                    if let Some(t) = http_tls {
                        cfg = cfg.with_tls(t);
                    }
                    let cfg = cfg;
                    match HttpServer::bind(http_reg, cfg).await {
                        Ok(srv) => {
                            eprintln!(
                                "  http      : {}://{}",
                                if srv.is_tls() { "https" } else { "http" },
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
    //
    // gRPC writes go straight to the engine, so it must never be an open side
    // door: once the wire listener is protected by `--secret`, or gRPC is bound
    // beyond loopback, it only starts with a bearer token (`--grpc-token`).
    let grpc_token = args
        .grpc_token
        .clone()
        .or_else(|| std::env::var("VECTORVOLT_GRPC_TOKEN").ok())
        .filter(|t| !t.is_empty());
    let grpc_needs_token =
        args.secret.is_some() || args.key_file.is_some() || !args.grpc_host.is_loopback();
    let grpc_blocked = grpc_needs_token && grpc_token.is_none();
    if grpc_blocked && !args.no_grpc && args.grpc_port > 0 {
        eprintln!(
            "  grpc      : disabled — set --grpc-token (or VECTORVOLT_GRPC_TOKEN) to enable it \
             alongside --secret or a non-loopback --grpc-host"
        );
    }
    let mut grpc_shutdown = if !args.no_grpc
        && !grpc_blocked
        && args.grpc_port > 0
        && let Some(ref eng) = engine
    {
        let grpc_addr = SocketAddr::new(args.grpc_host, args.grpc_port);
        let grpc_engine = Arc::clone(eng);
        use vectorvolt_server::GrpcServer;
        let mut grpc_srv = GrpcServer::standalone(grpc_addr, grpc_engine);
        grpc_srv = grpc_srv
            .with_write_gate(gate_w.clone())
            .with_after_write(gate_a.clone());
        if let Some(token) = grpc_token {
            grpc_srv = grpc_srv.with_token(token);
        }
        if let Some(t) = &tls {
            grpc_srv = grpc_srv.with_tls_pem(&t.cert_pem, &t.key_pem, t.client_ca_pem.as_deref());
        }
        warn_plaintext("gRPC", grpc_addr, grpc_srv.is_tls());
        let grpc_scheme = if grpc_srv.is_tls() { "grpcs" } else { "grpc" };
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        std::thread::Builder::new()
            .name("grpc-server".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    eprintln!("  grpc      : {grpc_scheme}://{grpc_addr}");
                    if let Err(e) = grpc_srv.run(rx).await {
                        eprintln!("gRPC server error: {e}");
                    }
                });
            })
            .expect("spawn grpc-server thread");
        Some(tx)
    } else {
        if !grpc_blocked {
            eprintln!("  grpc      : disabled");
        }
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
        // D3: `--pgwire-host` (default loopback) instead of a hardcoded 127.0.0.1.
        let pgwire_addr = pgwire_addr(&args, pgwire_port);
        // D1: pgwire answers SSLRequest with 'S' and upgrades when TLS is set.
        let pg_tls = tls.as_ref().map(|t| t.net.rustls_config());
        let pg_require_tls = args.listener.pgwire_require_tls;
        let pg_max_connections = args.listener.pgwire_max_connections;
        let pg_auth = args.listener.pgwire_auth;
        warn_plaintext("pgwire", pgwire_addr, pg_tls.is_some());
        let pg_dir = args.dir.clone();
        let (pg_gate_w, pg_gate_a) = (gate_w.clone(), gate_a.clone());
        // A3: `--secret` also gates pgwire (clients send it as their password).
        let pg_password = args.secret.as_ref().map(|s| s.trim().to_owned());
        // A2: SQL column encryption uses a key derived from the master key.
        let pg_column_key = master_key.as_deref().map(derive_column_key).transpose()?;
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        pgwire_shutdown = Some(tx);
        std::thread::Builder::new()
            .name("pgwire-server".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    let config = PgConfig::new(pgwire_addr, pg_dir);
                    let pg_tls_on = pg_tls.is_some();
                    let server =
                        build_pg_server(config, pg_reg, pg_password, pg_auth, pg_column_key)
                            .with_tls(pg_tls)
                            .with_write_gate(pg_gate_w)
                            .with_after_write(pg_gate_a)
                            .with_require_tls(pg_require_tls)
                            .with_max_connections(pg_max_connections);
                    eprintln!(
                        "  pgwire    : postgres://{pgwire_addr}{}",
                        match (pg_tls_on, pg_require_tls) {
                            (true, true) => " (TLS required)",
                            (true, false) => " (TLS offered)",
                            _ => "",
                        }
                    );
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

    // ── Cluster node (optional) ──────────────────────────────────────────────
    // Real SWIM gossip over UDP on --cluster-bind, joining via
    // --cluster-seeds; membership and the vvctl control path (status /
    // members / join / leave) are served over HTTP on --cluster-status-port.
    if let Some(cluster_bind) = args.cluster_bind {
        let node_id = match args.cluster_node_id.as_deref() {
            Some(s) => match uuid::Uuid::parse_str(s) {
                Ok(u) => Some(NodeId(u)),
                Err(e) => {
                    eprintln!("invalid --cluster-node-id {s:?}: {e}");
                    None
                }
            },
            None => super::cluster_cmd::load_identity(&args.dir).map(|i| NodeId(i.node_id)),
        };
        let cluster_node = ClusterNode::start(cluster_bind, args.cluster_seeds.clone(), node_id);
        // `vvctl cluster join/leave` and gossip join/leave events drive the
        // failover Raft group's membership.
        if let Some(fh) = &_failover {
            cluster_node.set_membership_hook(Arc::new(
                super::serve_failover::RaftMembershipBridge::new(
                    Arc::clone(&fh.node.controller),
                    fh.seeds.clone(),
                ),
            ));
        }
        eprintln!(
            "  cluster   : node {} on {} ({} seed(s))",
            cluster_node.status().node_id,
            cluster_bind,
            args.cluster_seeds.len()
        );
        let status_port = args.cluster_status_port;
        // `vvctl cluster status/members/join/leave` find this node's control
        // endpoint through the identity file instead of needing `--node`.
        if let Err(e) = super::cluster_cmd::record_control_addr(
            &args.dir,
            SocketAddr::from(([127, 0, 0, 1], status_port)),
        ) {
            eprintln!("  cluster   : could not record the control address: {e}");
        }
        std::thread::Builder::new()
            .name("cluster-status".into())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    if let Err(e) = cluster_node.start_gossip(Default::default()).await {
                        eprintln!("  cluster   : gossip bind failed on {cluster_bind}: {e}");
                    }
                    eprintln!("  cluster   : status http://127.0.0.1:{status_port}/cluster/status");
                    ClusterStatusServer::new(cluster_node)
                        .serve(status_port)
                        .await;
                });
            })
            .expect("spawn cluster-status thread");
    } else {
        eprintln!(
            "  cluster   : disabled (pass a cluster-bind address after the table dir to enable)"
        );
    }

    // ── WAL-shipping replication (F5) ────────────────────────────────────────
    let _replication = start_replication(&args, engine.as_ref(), master_key.as_ref())?;

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
        let metrics_addr = SocketAddr::new(args.metrics_host, args.metrics_port);
        std::thread::Builder::new()
            .name("metrics-http".into())
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(async move {
                    MetricsServer::new(metrics_reg).serve_on(metrics_addr).await;
                });
            })
            .expect("spawn metrics-http thread");
        eprintln!("  metrics   : http://{metrics_addr}/metrics");
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

/// Live replication endpoint started by `--replication-role`. Dropping it
/// stops replication; it owns the runtime its tasks run on.
pub struct ReplicationHandle {
    _rt: tokio::runtime::Runtime,
    pub server: Option<Arc<ReplicationServer>>,
    pub client: Option<Arc<ReplicationClient>>,
}

/// Validate the replication flags and start the leader listener or follower
/// pull loop. Returns `Ok(None)` when replication is not requested.
pub fn start_replication(
    args: &ServeArgs,
    engine: Option<&Arc<TableEngine>>,
    key: Option<&Arc<KeyStore>>,
) -> Result<Option<ReplicationHandle>> {
    let r = &args.replication;
    let Some(role) = r.replication_role else {
        // `--failover` owns `--replication-listen` (the fenced front).
        if r.failover {
            return Ok(None);
        }
        if r.replicate_from.is_some() || r.replication_listen.is_some() {
            bail!("--replicate-from / --replication-listen require --replication-role");
        }
        eprintln!("  replication: disabled (use --replication-role to enable)");
        return Ok(None);
    };
    if role == ReplicationRole::Standalone {
        return Ok(None);
    }
    let engine = match (args.multi, engine) {
        (false, Some(e)) => Arc::clone(e),
        _ => bail!("--replication-role supports single-table mode only (not --multi)"),
    };
    let cfg = ReplicationConfig::default()
        .with_mode(r.replication_mode)
        .with_replication_factor(r.replication_factor);
    cfg.validate()?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("replication")
        .enable_all()
        .build()
        .with_context(|| "build replication runtime")?;
    let cell = RoleCell::new(role);
    match role {
        ReplicationRole::Leader => {
            if r.replicate_from.is_some() {
                bail!("--replicate-from is for followers; a leader uses --replication-listen");
            }
            let listen = r.replication_listen.ok_or_else(|| {
                anyhow::anyhow!("--replication-role leader needs --replication-listen")
            })?;
            let wal_dir = args.dir.join("wal");
            let server = rt
                .block_on(ReplicationServer::start(engine, wal_dir, cfg, cell, listen))
                .with_context(|| format!("start replication listener on {listen}"))?;
            eprintln!(
                "  replication: leader on {} ({}, factor {})",
                server.local_addr(),
                r.replication_mode,
                r.replication_factor
            );
            Ok(Some(ReplicationHandle {
                _rt: rt,
                server: Some(server),
                client: None,
            }))
        }
        ReplicationRole::Follower => {
            if r.replication_listen.is_some() {
                bail!("--replication-listen is for leaders; a follower uses --replicate-from");
            }
            let from = r.replicate_from.clone().ok_or_else(|| {
                anyhow::anyhow!("--replication-role follower needs --replicate-from")
            })?;
            // A follower must not take client writes: the only credential it
            // accepts is a read-only one.
            if args.secret.is_none()
                || args.key_scope != Some(KeyScope::ReadOnly)
                || args.key_file.is_some()
            {
                bail!(
                    "a replication follower is read-only: pass --secret with --key-scope \
                     read-only (and no --key-file) so clients cannot write to it"
                );
            }
            let client = {
                let _g = rt.enter();
                ReplicationClient::start(engine, &args.dir, key.cloned(), cfg, cell, from.clone())
                    .with_context(|| "start replication follower")?
            };
            eprintln!(
                "  replication: follower of {from} (applied lsn {}, read-only)",
                client.applied_lsn()
            );
            Ok(Some(ReplicationHandle {
                _rt: rt,
                server: None,
                client: Some(client),
            }))
        }
        ReplicationRole::Standalone => Ok(None),
    }
}

#[cfg(test)]
mod replication_flag_tests {
    use super::*;

    fn args() -> ServeArgs {
        ServeArgs {
            dir: PathBuf::from("."),
            ..ServeArgs::default()
        }
    }

    #[test]
    fn replication_is_off_by_default_and_flags_need_a_role() {
        assert!(start_replication(&args(), None, None).unwrap().is_none());
        let mut a = args();
        a.replication.replicate_from = Some("127.0.0.1:1".into());
        assert!(start_replication(&a, None, None).is_err());
    }

    #[test]
    fn follower_must_be_read_only_and_leader_needs_a_listener() {
        let dir = tempfile::TempDir::new().unwrap();
        let engine = Arc::new(
            TableEngine::open_with(vec!["pk".into()], dir.path(), EngineOptions::new(1024))
                .unwrap(),
        );
        let mut a = args();
        a.dir = dir.path().to_path_buf();
        a.replication.replication_role = Some(ReplicationRole::Follower);
        a.replication.replicate_from = Some("127.0.0.1:1".into());
        let err = start_replication(&a, Some(&engine), None).err().unwrap();
        assert!(err.to_string().contains("read-only"), "{err}");

        a.replication.replication_role = Some(ReplicationRole::Leader);
        a.replication.replicate_from = None;
        let err = start_replication(&a, Some(&engine), None).err().unwrap();
        assert!(err.to_string().contains("--replication-listen"), "{err}");
    }
}

#[cfg(test)]
mod key_scope_tests {
    use super::*;

    const HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    #[test]
    fn multi_defaults_the_secret_key_to_admin() {
        let args = ServeArgs {
            secret: Some(HEX.into()),
            multi: true,
            ..ServeArgs::default()
        };
        assert_eq!(
            build_auth(&args).unwrap().scope_of("default"),
            Some(Scope::Admin)
        );
    }

    #[test]
    fn single_table_keeps_read_write_and_key_scope_overrides() {
        let mut args = ServeArgs {
            secret: Some(HEX.into()),
            ..ServeArgs::default()
        };
        assert_eq!(
            build_auth(&args).unwrap().scope_of("default"),
            Some(Scope::ReadWrite)
        );
        args.key_scope = Some(KeyScope::ReadOnly);
        assert_eq!(
            build_auth(&args).unwrap().scope_of("default"),
            Some(Scope::ReadOnly)
        );
    }

    #[test]
    fn key_file_adds_scoped_keys_and_rejects_bad_lines() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("keys");
        std::fs::write(
            &path,
            format!(
                "# keys

reader read-only {HEX}
ops admin {HEX}
"
            ),
        )
        .unwrap();
        let args = ServeArgs {
            key_file: Some(path.clone()),
            ..ServeArgs::default()
        };
        let auth = build_auth(&args).unwrap();
        assert_eq!(auth.scope_of("reader"), Some(Scope::ReadOnly));
        assert_eq!(auth.scope_of("ops"), Some(Scope::Admin));

        std::fs::write(
            &path,
            format!(
                "ops superuser {HEX}
"
            ),
        )
        .unwrap();
        assert!(build_auth(&args).is_err());
        std::fs::write(
            &path,
            "ops admin
",
        )
        .unwrap();
        assert!(build_auth(&args).is_err());
    }
}

/// Live failover node started by `--failover`. Dropping it stops the control
/// plane; it owns the runtime its tasks run on.
pub struct FailoverHandle {
    _rt: tokio::runtime::Runtime,
    pub node: Arc<super::serve_failover::FailoverNode>,
    /// Replication addresses of the configured `--failover-peer` members.
    pub seeds: Vec<SocketAddr>,
}

/// Start the epoch-fenced failover control plane when `--failover` is set and
/// install the resulting fences into the listeners' late-bound gates.
pub fn start_failover(
    args: &ServeArgs,
    engine: Option<&Arc<TableEngine>>,
    key: Option<&Arc<KeyStore>>,
    client_addr: SocketAddr,
    write_gate: &super::serve_failover::LateGate,
    after_write: &super::serve_failover::LateGate,
) -> Result<Option<FailoverHandle>> {
    use super::serve_failover::{FailoverNode, FailoverNodeOpts, parse_node_id, parse_peer};
    use vectorvolt_cluster::failover::FailoverConfig;

    let r = &args.replication;
    if !r.failover {
        if r.raft_listen.is_some() || !r.failover_peers.is_empty() {
            bail!("--raft-listen / --failover-peer require --failover");
        }
        return Ok(None);
    }
    if r.replication_role.is_some() || r.replicate_from.is_some() {
        bail!("--failover elects the primary itself; drop --replication-role / --replicate-from");
    }
    let engine = match (args.multi, engine) {
        (false, Some(e)) => Arc::clone(e),
        _ => bail!("--failover supports single-table mode only (not --multi)"),
    };
    let raft_listen = r
        .raft_listen
        .ok_or_else(|| anyhow::anyhow!("--failover needs --raft-listen"))?;
    let repl_listen = r
        .replication_listen
        .ok_or_else(|| anyhow::anyhow!("--failover needs --replication-listen"))?;
    let node_id = match args.cluster_node_id.as_deref() {
        Some(s) => {
            parse_node_id(s).map_err(|e| anyhow::anyhow!("invalid --cluster-node-id: {e}"))?
        }
        None => super::cluster_cmd::load_identity(&args.dir)
            .map(|i| NodeId(i.node_id))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--failover needs a stable node id: pass --cluster-node-id or run \
                     `vvctl cluster init`"
                )
            })?,
    };
    let mut peers = Vec::new();
    for p in &r.failover_peers {
        peers.push(parse_peer(p).map_err(|e| anyhow::anyhow!(e))?);
    }
    let mut control = FailoverConfig::new(
        node_id,
        args.dir.join("failover"),
        peers.iter().map(|p| p.0.clone()).collect(),
    );
    control.raft_listen = raft_listen;
    control.repl_listen = repl_listen;
    control.join = r.failover_join;
    control.admin_token = args.secret.clone();
    control.client_addr = r
        .failover_client_addr
        .clone()
        .unwrap_or_else(|| client_addr.to_string());
    let repl = ReplicationConfig::default()
        .with_mode(r.replication_mode)
        .with_replication_factor(r.replication_factor)
        .with_degraded_ok(r.replication_degraded_ok);
    repl.validate()?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("failover")
        .enable_all()
        .build()
        .with_context(|| "build failover runtime")?;
    let node = rt
        .block_on(FailoverNode::start(FailoverNodeOpts {
            engine,
            dir: args.dir.clone(),
            key: key.cloned(),
            repl,
            control,
        }))
        .with_context(|| "start failover node")?;
    let seeds: Vec<SocketAddr> = peers.iter().map(|p| p.2).collect();
    for (id, raft, repl_addr, client) in peers {
        node.controller.add_peer(&id, raft, repl_addr, client);
    }
    if r.failover_join {
        let _guard = rt.enter();
        node.controller.auto_join(seeds.clone());
    }
    write_gate.set(node.write_gate());
    after_write.set(node.after_write());
    {
        let fence = node.fence();
        // Commit versionstamps carry the failover epoch.
        super::serve_cluster_ext::set_epoch_source(Arc::new(move || fence.epoch()));
    }
    eprintln!(
        "  failover  : node {} raft {} replication {} ({}, factor {}); read-only until elected",
        node.controller.node_id(),
        node.controller.raft_addr(),
        node.controller.repl_addr(),
        r.replication_mode,
        r.replication_factor
    );
    Ok(Some(FailoverHandle {
        _rt: rt,
        node,
        seeds,
    }))
}
