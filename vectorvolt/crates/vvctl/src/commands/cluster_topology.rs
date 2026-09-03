//! `vvctl cluster-topology` — show cluster node/shard distribution (VV-CTL-005).

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Leader,
    Follower,
    Dead,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Leader => write!(f, "LEADER"),
            NodeRole::Follower => write!(f, "FOLLOWER"),
            NodeRole::Dead => write!(f, "DEAD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub addr: String,
    pub role: NodeRole,
    pub shard_count: usize,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ClusterOutputFormat {
    Tree,
    Json,
    Table,
}

#[derive(Args)]
pub struct ClusterTopologyArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "tree")]
    pub output: ClusterOutputFormat,
    /// Cluster name for display.
    #[arg(long, default_value = "vectorvolt-cluster")]
    pub cluster_name: String,
}

pub fn format_tree(nodes: &[NodeInfo], total_shards: usize) -> String {
    if nodes.is_empty() {
        return "Cluster: (empty — no nodes registered)\n".to_string();
    }
    let mut out = format!(
        "Cluster: vectorvolt-cluster ({} nodes, {} total shards)\n",
        nodes.len(),
        total_shards
    );
    let last = nodes.len() - 1;
    for (i, node) in nodes.iter().enumerate() {
        let prefix = if i == last {
            "└── "
        } else {
            "├── "
        };
        out.push_str(&format!(
            "{}{} [{}] addr={} shards={}/{}\n",
            prefix, node.id, node.role, node.addr, node.shard_count, total_shards
        ));
    }
    out
}

pub fn format_json(nodes: &[NodeInfo]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({ "nodes": nodes }))
        .unwrap_or_else(|_| "{}".to_string())
}

pub fn format_table(nodes: &[NodeInfo]) -> String {
    let mut out = format!(
        "{:<20} {:<12} {:<25} {:<10}\n",
        "ID", "ROLE", "ADDR", "SHARDS"
    );
    out.push_str(&"-".repeat(70));
    out.push('\n');
    for node in nodes {
        out.push_str(&format!(
            "{:<20} {:<12} {:<25} {:<10}\n",
            node.id,
            node.role.to_string(),
            node.addr,
            node.shard_count
        ));
    }
    out
}

pub fn default_nodes() -> Vec<NodeInfo> {
    vec![
        NodeInfo {
            id: "node-1".into(),
            addr: "127.0.0.1:50051".into(),
            role: NodeRole::Leader,
            shard_count: 1365,
        },
        NodeInfo {
            id: "node-2".into(),
            addr: "127.0.0.2:50051".into(),
            role: NodeRole::Follower,
            shard_count: 1365,
        },
        NodeInfo {
            id: "node-3".into(),
            addr: "127.0.0.3:50051".into(),
            role: NodeRole::Follower,
            shard_count: 1366,
        },
    ]
}

pub fn run(args: ClusterTopologyArgs) -> Result<()> {
    let nodes = default_nodes();
    let total: usize = nodes.iter().map(|n| n.shard_count).sum();
    let out = match args.output {
        ClusterOutputFormat::Tree => format_tree(&nodes, total),
        ClusterOutputFormat::Json => format_json(&nodes),
        ClusterOutputFormat::Table => format_table(&nodes),
    };
    print!("{out}");
    Ok(())
}
