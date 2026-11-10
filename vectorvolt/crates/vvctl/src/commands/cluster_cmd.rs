//! `vvctl cluster` — manage a VectorVolt cluster (VV-CLU-CLI-001..006).
//!
//! Two modes:
//! - Live (default for `status`, `members`/`nodes`, `join`, `leave`): talk to
//!   the running `vvctl serve --cluster-bind` node's control endpoint, which
//!   reports its real SWIM membership, joins through a seed, or gracefully
//!   leaves (broadcasting Left). The endpoint is `--node host:port` if given,
//!   else the `control_addr` recorded in `<dir>/cluster_identity.json`, else
//!   the local default `127.0.0.1:7900` (serve's default `--cluster-status-port`).
//! - Offline (`--offline` only): read/write the static `cluster.json` in the
//!   data dir. Never selected implicitly; `init` and `ring` are always local.
//!
//! `init` also writes `cluster_identity.json` (fresh cluster id + node id)
//! which `vvctl serve` uses as its node id.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use vectorvolt_cluster::config::ClusterConfig;
use vectorvolt_cluster::node::{NodeId, NodeMeta, NodeState};

const DEFAULT_DIR: &str = ".";
const STATE_FILE: &str = "cluster.json";

#[derive(Args)]
pub struct ClusterArgs {
    /// Data directory containing cluster.json (default: current dir).
    #[arg(short, long, default_value = DEFAULT_DIR)]
    pub dir: PathBuf,

    /// Control endpoint of a running node (its `--cluster-status-port`).
    /// When set, status/members/join/leave act on the live cluster.
    #[arg(long)]
    pub node: Option<SocketAddr>,

    /// Operate on the static `cluster.json` instead of the running node.
    /// Without it, `status`/`members`/`join`/`leave` talk to the local node.
    #[arg(long, conflicts_with = "node")]
    pub offline: bool,

    #[command(subcommand)]
    pub command: ClusterCommand,
}

/// Cluster/node identity created by `cluster init`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterIdentity {
    pub cluster_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub cluster_name: String,
    pub created_unix_secs: u64,
    /// Control endpoint (`--cluster-status-port`) of this node, recorded by
    /// `serve`; absent in identities written before it was tracked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_addr: Option<SocketAddr>,
}

const IDENTITY_FILE: &str = "cluster_identity.json";

/// `serve`'s default `--cluster-status-port`, used when the identity does not
/// record a control address.
pub const DEFAULT_CONTROL_ADDR: &str = "127.0.0.1:7900";

/// Load `<dir>/cluster_identity.json` if present and valid.
pub fn load_identity(dir: &Path) -> Option<ClusterIdentity> {
    let raw = std::fs::read_to_string(dir.join(IDENTITY_FILE)).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Record this node's control endpoint in `<dir>/cluster_identity.json` so
/// `vvctl cluster status/members/join/leave` find it without `--node`.
/// No-op if the directory has no identity.
pub fn record_control_addr(dir: &Path, addr: SocketAddr) -> Result<()> {
    let Some(mut id) = load_identity(dir) else {
        return Ok(());
    };
    if id.control_addr != Some(addr) {
        id.control_addr = Some(addr);
        std::fs::write(dir.join(IDENTITY_FILE), serde_json::to_string_pretty(&id)?)?;
    }
    Ok(())
}

/// Which live endpoint a command should talk to; `None` means run offline.
pub fn resolve_target(args: &ClusterArgs) -> Option<SocketAddr> {
    if let Some(n) = args.node {
        return Some(n);
    }
    let live_capable = matches!(
        args.command,
        ClusterCommand::Status
            | ClusterCommand::Nodes
            | ClusterCommand::Members
            | ClusterCommand::Join { .. }
            | ClusterCommand::Leave { .. }
    );
    if args.offline || !live_capable {
        return None;
    }
    Some(
        load_identity(&args.dir)
            .and_then(|i| i.control_addr)
            .unwrap_or_else(|| DEFAULT_CONTROL_ADDR.parse().expect("valid default")),
    )
}

/// Minimal HTTP/1.1 client for the node control endpoint.
fn http(node: SocketAddr, method: &str, path: &str) -> Result<String> {
    use std::io::{Read, Write};
    let timeout = std::time::Duration::from_secs(10);
    let mut s = std::net::TcpStream::connect_timeout(&node, timeout).with_context(|| {
        format!(
            "cannot reach cluster node at {node} (is `serve --cluster-bind` running? \nuse --node host:port for another endpoint, or --offline for cluster.json)"
        )
    })?;
    s.set_read_timeout(Some(timeout))?;
    let crlf = "\r\n";
    let req =
        format!("{method} {path} HTTP/1.1{crlf}Host: {node}{crlf}Connection: close{crlf}{crlf}");
    s.write_all(req.as_bytes())?;
    let mut raw = String::new();
    s.read_to_string(&mut raw)?;
    let (head, body) = raw
        .split_once(&format!("{crlf}{crlf}"))
        .unwrap_or((&raw, ""));
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    if code != 200 {
        anyhow::bail!("node returned {code}: {}", body.trim());
    }
    Ok(body.to_string())
}

/// Run `cmd` against the live node; `Ok(false)` if the command is offline-only.
fn live(node: SocketAddr, cmd: &ClusterCommand) -> Result<bool> {
    use vectorvolt_cluster::node_runtime::{ClusterStatus, PeerInfo};
    match cmd {
        ClusterCommand::Status => {
            let st: ClusterStatus = serde_json::from_str(&http(node, "GET", "/cluster/status")?)?;
            println!("Node:     {} ({})", st.node_id, st.bind_addr);
            println!(
                "Members:  {} total ({} alive, {} suspect, {} dead, {} left)",
                st.membership_size, st.alive, st.suspect, st.dead, st.left
            );
            let gossip = if st.gossip_active {
                "active"
            } else {
                "not started"
            };
            println!("Gossip:   {gossip}");
            println!("Seeds:    {}", st.configured_seeds.join(", "));
            Ok(true)
        }
        ClusterCommand::Nodes | ClusterCommand::Members => {
            let ms: Vec<PeerInfo> = serde_json::from_str(&http(node, "GET", "/cluster/members")?)?;
            println!(
                "{:<38} {:<22} {:<10} INCARNATION",
                "NODE-ID", "ADDRESS", "STATE"
            );
            println!("{}", "-".repeat(80));
            for m in ms {
                println!(
                    "{:<38} {:<22} {:<10} {}",
                    m.node_id,
                    m.addr,
                    m.state.to_lowercase(),
                    m.incarnation
                );
            }
            Ok(true)
        }
        ClusterCommand::Join { addr } => {
            http(node, "POST", &format!("/cluster/join?seed={addr}"))?;
            println!("Node {node} joined the cluster via seed {addr}.");
            Ok(true)
        }
        ClusterCommand::Leave { .. } => {
            http(node, "POST", "/cluster/leave")?;
            println!("Node {node} left the cluster (Left broadcast).");
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[derive(Subcommand)]
pub enum ClusterCommand {
    /// Initialize a new cluster config in the data directory.
    Init {
        /// Cluster name.
        #[arg(long, default_value = "vectorvolt")]
        name: String,
        /// Replication factor (default 3).
        #[arg(long, default_value_t = 3)]
        replication_factor: usize,
        /// Number of vnodes per node (default 4096).
        #[arg(long, default_value_t = 4096)]
        vnode_count: usize,
    },

    /// List all nodes in the cluster.
    Nodes,

    /// List cluster members (live by default; `--offline` reads cluster.json).
    Members,

    /// Show overall cluster health (alive / suspect / dead counts).
    Status,

    /// Add a node to the cluster state.
    Join {
        /// Node address (host:port).
        #[arg(long)]
        addr: SocketAddr,
    },

    /// Gracefully make the local node leave (live), or with `--offline` mark a
    /// node as Left by its address in cluster.json.
    Leave {
        /// Node address to remove (required offline).
        #[arg(long)]
        addr: Option<SocketAddr>,
    },

    /// Show vnode ring ownership summary.
    Ring {
        /// Sample key to look up owner for.
        #[arg(long)]
        key: Option<u64>,
    },
}

fn state_path(dir: &Path) -> PathBuf {
    dir.join(STATE_FILE)
}

fn load(dir: &Path) -> Result<ClusterConfig> {
    let p = state_path(dir);
    let raw = std::fs::read_to_string(&p).with_context(|| {
        format!(
            "no cluster.json in {} — run `vvctl cluster init` first",
            dir.display()
        )
    })?;
    serde_json::from_str(&raw).context("cluster.json is malformed")
}

fn save(dir: &Path, cfg: &ClusterConfig) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(cfg)?;
    std::fs::write(state_path(dir), json)?;
    Ok(())
}

pub fn run(args: ClusterArgs) -> Result<()> {
    if let Some(node) = resolve_target(&args)
        && live(node, &args.command)?
    {
        return Ok(());
    }
    match args.command {
        ClusterCommand::Init {
            name,
            replication_factor,
            vnode_count,
        } => {
            let mut cfg = ClusterConfig::new(&name);
            cfg.replication_factor = replication_factor;
            cfg.vnode_count = vnode_count;
            save(&args.dir, &cfg)?;
            let ident = ClusterIdentity {
                cluster_id: uuid::Uuid::new_v4(),
                node_id: uuid::Uuid::new_v4(),
                cluster_name: name.clone(),
                created_unix_secs: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                control_addr: None,
            };
            std::fs::write(
                args.dir.join(IDENTITY_FILE),
                serde_json::to_string_pretty(&ident)?,
            )?;
            println!(
                "Initialized cluster '{}' (id {}, node {}) in {}",
                name,
                ident.cluster_id,
                ident.node_id,
                args.dir.display()
            );
        }

        ClusterCommand::Nodes | ClusterCommand::Members => {
            let cfg = load(&args.dir)?;
            if cfg.nodes.is_empty() {
                println!("No nodes. Use `vvctl cluster join --addr <host:port>` to add one.");
                return Ok(());
            }
            println!(
                "{:<38} {:<22} {:<10} INCARNATION",
                "NODE-ID", "ADDRESS", "STATE"
            );
            println!("{}", "-".repeat(80));
            for n in &cfg.nodes {
                println!(
                    "{:<38} {:<22} {:<10} {}",
                    n.id.0,
                    n.addr,
                    state_label(&n.state),
                    n.incarnation,
                );
            }
        }

        ClusterCommand::Status => {
            let cfg = load(&args.dir)?;
            let alive = cfg
                .nodes
                .iter()
                .filter(|n| n.state == NodeState::Alive)
                .count();
            let suspect = cfg
                .nodes
                .iter()
                .filter(|n| n.state == NodeState::Suspect)
                .count();
            let dead = cfg
                .nodes
                .iter()
                .filter(|n| n.state == NodeState::Dead)
                .count();
            let left = cfg
                .nodes
                .iter()
                .filter(|n| n.state == NodeState::Left)
                .count();
            let quorum = alive >= cfg.write_quorum;
            println!("Cluster:  {}", cfg.cluster_name);
            println!(
                "Nodes:    {} total ({alive} alive, {suspect} suspect, {dead} dead, {left} left)",
                cfg.nodes.len()
            );
            println!(
                "RF:       {}  write_q={}  read_q={}",
                cfg.replication_factor, cfg.write_quorum, cfg.read_quorum
            );
            println!("Vnodes:   {}", cfg.vnode_count);
            println!(
                "Quorum:   {}",
                if quorum {
                    "OK"
                } else {
                    "DEGRADED — not enough alive nodes for writes"
                }
            );
        }

        ClusterCommand::Join { addr } => {
            let mut cfg = load(&args.dir).unwrap_or_else(|_| ClusterConfig::new("vectorvolt"));
            if cfg.nodes.iter().any(|n| n.addr == addr) {
                println!("Node {} is already in the cluster.", addr);
                return Ok(());
            }
            let now_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            cfg.nodes.push(NodeMeta {
                id: NodeId::new(),
                addr,
                state: NodeState::Alive,
                incarnation: 1,
                last_seen_ns: now_ns,
            });
            save(&args.dir, &cfg)?;
            println!("Joined {} into cluster '{}'.", addr, cfg.cluster_name);
        }

        ClusterCommand::Leave { addr } => {
            let addr = addr.context(
                "--addr is required with --offline (without it, leave acts on the live node)",
            )?;
            let mut cfg = load(&args.dir)?;
            let mut found = false;
            for n in cfg.nodes.iter_mut() {
                if n.addr == addr {
                    n.state = NodeState::Left;
                    found = true;
                }
            }
            if !found {
                anyhow::bail!("No node with address {} found.", addr);
            }
            save(&args.dir, &cfg)?;
            println!("Marked {} as Left.", addr);
        }

        ClusterCommand::Ring { key } => {
            let cfg = load(&args.dir)?;
            let mut ring = vectorvolt_cluster::ring::HashRing::new(cfg.vnode_count);
            for n in &cfg.nodes {
                if n.state == NodeState::Alive {
                    ring.add_node(n.id.clone());
                }
            }
            let alive = cfg
                .nodes
                .iter()
                .filter(|n| n.state == NodeState::Alive)
                .count();
            println!(
                "Ring: {} vnodes, {} alive nodes",
                cfg.vnode_count * alive.max(1),
                alive
            );
            if let Some(k) = key {
                match ring.owner_of(k) {
                    Some(id) => {
                        if let Some(n) = cfg.nodes.iter().find(|n| n.id == id) {
                            println!("Key {} → node {} ({})", k, id.0, n.addr);
                        } else {
                            println!("Key {} → node {}", k, id.0);
                        }
                    }
                    None => println!("Ring is empty — no alive nodes."),
                }
            }
        }
    }
    Ok(())
}

fn state_label(s: &NodeState) -> &'static str {
    match s {
        NodeState::Alive => "alive",
        NodeState::Suspect => "suspect",
        NodeState::Dead => "dead",
        NodeState::Left => "left",
    }
}
