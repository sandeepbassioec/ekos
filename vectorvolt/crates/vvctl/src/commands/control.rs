//! `vvctl control` — control plane subcommands (VV-CTL-006).

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use vectorvolt_control::{BackendError, ControlBackend};

#[derive(Subcommand)]
pub enum ControlCommand {
    /// Start the control plane HTTP server.
    Start(ControlStartArgs),
    /// Query the control plane health endpoint.
    Status(ControlStatusArgs),
    /// List cluster nodes.
    Nodes(ControlNodesArgs),
    /// List cluster shards.
    Shards(ControlShardsArgs),
    /// Trigger a snapshot for a table.
    Snapshot(ControlSnapshotArgs),
}

#[derive(Args)]
pub struct ControlStartArgs {
    /// Bind address for the control server.
    #[arg(long, default_value = "127.0.0.1:18080")]
    pub bind: String,
    /// Bearer token clients must present. Falls back to `VV_CONTROL_TOKEN`;
    /// when neither is set a random token is generated and printed once.
    #[arg(long)]
    pub token: Option<String>,
    /// Data directory of the tables the API reports on. Without it the server
    /// has no engine attached and table/cluster endpoints answer 501.
    #[arg(long)]
    pub dir: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct ControlStatusArgs {
    /// Base URL of the control server.
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    /// Bearer token (optional: `/health` is unauthenticated). Falls back to
    /// `VV_CONTROL_TOKEN`.
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args)]
pub struct ControlNodesArgs {
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    /// Bearer token (required; falls back to `VV_CONTROL_TOKEN`).
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args)]
pub struct ControlShardsArgs {
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    /// Bearer token (required; falls back to `VV_CONTROL_TOKEN`).
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args)]
pub struct ControlSnapshotArgs {
    /// Table name to snapshot.
    pub table: String,
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    /// Bearer token (required; falls back to `VV_CONTROL_TOKEN`).
    #[arg(long)]
    pub token: Option<String>,
}

/// Environment variable holding the control-plane bearer token.
pub const TOKEN_ENV: &str = "VV_CONTROL_TOKEN";

/// Pick a non-empty token: the flag wins over the environment value.
fn pick_token(flag: Option<&str>, env: Option<String>) -> Option<String> {
    flag.map(str::to_owned)
        .or(env)
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// Token a client must send: `--token`, else `VV_CONTROL_TOKEN`, else an error.
fn client_token(flag: Option<&str>) -> Result<String> {
    pick_token(flag, std::env::var(TOKEN_ENV).ok())
        .with_context(|| format!("no control token: pass --token or set {TOKEN_ENV}"))
}

/// Random 256-bit token, hex-encoded.
fn generate_token() -> Result<String> {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).map_err(|e| anyhow::anyhow!("generate control token: {e}"))?;
    Ok(hex::encode(buf))
}

/// Token for the server: configured value, or a freshly generated one
/// (returned with `true`) when none was given.
fn server_token(flag: Option<&str>, env: Option<String>) -> Result<(String, bool)> {
    match pick_token(flag, env) {
        Some(t) => Ok((t, false)),
        None => Ok((generate_token()?, true)),
    }
}

/// Print the body of a 2xx response; any other status is an error (non-zero exit).
fn print_response(resp: reqwest::blocking::Response) -> Result<()> {
    let status = resp.status();
    let body = resp.text()?;
    if !status.is_success() {
        anyhow::bail!("control server returned {status}: {body}");
    }
    println!("{body}");
    Ok(())
}

/// Reports on a real [`TableRegistry`]: tables, per-table engine stats and
/// on-demand snapshots. Cluster data has no source here, so it stays 501.
pub struct RegistryBackend {
    registry: std::sync::Arc<vectorvolt_engine::TableRegistry>,
}

impl RegistryBackend {
    pub fn new(registry: std::sync::Arc<vectorvolt_engine::TableRegistry>) -> Self {
        Self { registry }
    }

    fn table(
        &self,
        name: &str,
    ) -> Result<std::sync::Arc<vectorvolt_engine::TableEngine>, BackendError> {
        if !self.registry.contains(name) {
            return Err(BackendError::NotFound(name.to_string()));
        }
        self.registry
            .get(name)
            .map_err(|e| BackendError::Failed(e.to_string()))
    }
}

impl ControlBackend for RegistryBackend {
    fn list_tables(&self) -> Vec<String> {
        self.registry.list_tables()
    }

    fn table_stats(&self, name: &str) -> Result<serde_json::Value, BackendError> {
        let s = self.table(name)?.stats();
        Ok(serde_json::json!({
            "table": name,
            "rows_buffered": s.rows_buffered,
            "rows_flushed": s.rows_flushed,
            "partition_count": s.partition_count,
            "columns": s.schema,
        }))
    }

    fn snapshot_table(&self, name: &str) -> Result<serde_json::Value, BackendError> {
        let m = self
            .table(name)?
            .take_snapshot()
            .map_err(|e| BackendError::Failed(e.to_string()))?;
        Ok(serde_json::json!({
            "table": name,
            "row_count": m.row_count,
            "byte_size": m.byte_size,
            "wal_seq": m.wal_seq,
        }))
    }
}

pub fn run_status(args: ControlStatusArgs) -> Result<()> {
    let _ = args.token; // `/health` needs no credentials
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/health", args.url))
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_nodes(args: ControlNodesArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/nodes", args.url))
        .header(
            "Authorization",
            format!("Bearer {}", client_token(args.token.as_deref())?),
        )
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_shards(args: ControlShardsArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/shards", args.url))
        .header(
            "Authorization",
            format!("Bearer {}", client_token(args.token.as_deref())?),
        )
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_snapshot(args: ControlSnapshotArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/tables/{}/snapshot", args.url, args.table))
        .header(
            "Authorization",
            format!("Bearer {}", client_token(args.token.as_deref())?),
        )
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_start(args: ControlStartArgs) -> Result<()> {
    use vectorvolt_control::{ControlConfig, ControlServer};
    let (token, generated) = server_token(args.token.as_deref(), std::env::var(TOKEN_ENV).ok())?;
    if generated {
        println!("Generated control token (shown once): {token}");
    }
    let cfg = ControlConfig {
        bind_addr: args.bind.clone(),
        token,
        users: vec![],
    };
    let server = match &args.dir {
        Some(dir) => {
            let registry = vectorvolt_engine::TableRegistry::open(
                dir,
                vectorvolt_engine::EngineOptions::default(),
            )
            .map_err(|e| anyhow::anyhow!("open registry {}: {e}", dir.display()))?;
            ControlServer::with_backend(
                cfg,
                std::sync::Arc::new(RegistryBackend::new(std::sync::Arc::new(registry))),
            )
        }
        None => ControlServer::new(cfg),
    };
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&args.bind).await?;
        println!("Control server listening on {}", args.bind);
        axum::serve(listener, server.router()).await?;
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}

pub fn run(cmd: ControlCommand) -> Result<()> {
    match cmd {
        ControlCommand::Start(args) => run_start(args),
        ControlCommand::Status(args) => run_status(args),
        ControlCommand::Nodes(args) => run_nodes(args),
        ControlCommand::Shards(args) => run_shards(args),
        ControlCommand::Snapshot(args) => run_snapshot(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_beats_env_and_blank_is_ignored() {
        assert_eq!(
            pick_token(Some("a"), Some("b".into())).as_deref(),
            Some("a")
        );
        assert_eq!(pick_token(None, Some("b".into())).as_deref(), Some("b"));
        assert_eq!(pick_token(None, Some("  ".into())), None);
        assert_eq!(pick_token(None, None), None);
    }

    #[test]
    fn server_generates_random_token_when_none_given() {
        let (a, ga) = server_token(None, None).unwrap();
        let (b, gb) = server_token(None, None).unwrap();
        assert!(ga && gb);
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert_ne!(a, "vvctl-token");
        let (t, g) = server_token(Some("given"), None).unwrap();
        assert_eq!((t.as_str(), g), ("given", false));
    }

    #[test]
    fn client_requires_a_token() {
        assert_eq!(client_token(Some("tok")).unwrap(), "tok");
        // Without the flag and without the env var there is no default.
        if std::env::var(TOKEN_ENV).is_err() {
            assert!(client_token(None).is_err());
        }
    }
}
