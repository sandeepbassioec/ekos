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
    /// Bearer token for authentication.
    #[arg(long, default_value = "vvctl-token")]
    pub token: String,
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
    /// Bearer token.
    #[arg(long, default_value = "vvctl-token")]
    pub token: String,
}

#[derive(Args)]
pub struct ControlNodesArgs {
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    #[arg(long, default_value = "vvctl-token")]
    pub token: String,
}

#[derive(Args)]
pub struct ControlShardsArgs {
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    #[arg(long, default_value = "vvctl-token")]
    pub token: String,
}

#[derive(Args)]
pub struct ControlSnapshotArgs {
    /// Table name to snapshot.
    pub table: String,
    #[arg(long, default_value = "http://127.0.0.1:18080")]
    pub url: String,
    #[arg(long, default_value = "vvctl-token")]
    pub token: String,
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
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/health", args.url))
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_nodes(args: ControlNodesArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/nodes", args.url))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_shards(args: ControlShardsArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/shards", args.url))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_snapshot(args: ControlSnapshotArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/tables/{}/snapshot", args.url, args.table))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    print_response(resp)
}

pub fn run_start(args: ControlStartArgs) -> Result<()> {
    use vectorvolt_control::{ControlConfig, ControlServer};
    let cfg = ControlConfig {
        bind_addr: args.bind.clone(),
        token: args.token.clone(),
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
