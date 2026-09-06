//! `vvctl cluster` — manage a VectorVolt cluster (VV-CLU-CLI-001..006).
//!
//! Commands read/write a `cluster.json` state file in the data directory.
//! No running daemon required — operations are in-process.

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

    #[command(subcommand)]
    pub command: ClusterCommand,
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

    /// Show overall cluster health (alive / suspect / dead counts).
    Status,

    /// Add a node to the cluster state.
    Join {
        /// Node address (host:port).
        #[arg(long)]
        addr: SocketAddr,
    },

    /// Mark a node as Left by its address.
    Leave {
        /// Node address to remove.
        #[arg(long)]
        addr: SocketAddr,
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
            println!("Initialized cluster '{}' in {}", name, args.dir.display());
        }

        ClusterCommand::Nodes => {
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
