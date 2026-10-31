//! `vvctl control` — control plane subcommands (VV-CTL-006).

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

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

pub fn run_status(args: ControlStatusArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/health", args.url))
        .send()
        .context("failed to reach control server")?;
    let body = resp.text()?;
    println!("{body}");
    Ok(())
}

pub fn run_nodes(args: ControlNodesArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/nodes", args.url))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    let body = resp.text()?;
    println!("{body}");
    Ok(())
}

pub fn run_shards(args: ControlShardsArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/cluster/shards", args.url))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    let body = resp.text()?;
    println!("{body}");
    Ok(())
}

pub fn run_snapshot(args: ControlSnapshotArgs) -> Result<()> {
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}/tables/{}/snapshot", args.url, args.table))
        .header("Authorization", format!("Bearer {}", args.token))
        .send()
        .context("failed to reach control server")?;
    let body = resp.text()?;
    println!("{body}");
    Ok(())
}

pub fn run_start(args: ControlStartArgs) -> Result<()> {
    use vectorvolt_control::{ControlConfig, ControlServer};
    let cfg = ControlConfig {
        bind_addr: args.bind.clone(),
        token: args.token.clone(),
        users: vec![],
    };
    let server = ControlServer::new(cfg);
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
