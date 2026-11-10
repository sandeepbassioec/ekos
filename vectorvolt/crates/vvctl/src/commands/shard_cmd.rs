//! `vvctl shard` — shard split, live migration, drain and rebalance (gap F7).
//!
//! Talks to a running node's control endpoint (`--node host:port`):
//!
//! - `GET  /shards`            -> `ShardTable` JSON
//! - `POST /shards/split`      `{"shard_id":N,"at":K?}`
//! - `POST /shards/move`       `{"shard_id":N,"to":"<node uuid>"}`
//! - `POST /shards/drain`      `{"node":"<node uuid>"}`
//! - `POST /shards/rebalance`  `{}`
//!
//! The node side is `vectorvolt_cluster::migration` (split/migrate/drain via
//! Raft-committed map changes) and `vectorvolt_cluster::rebalance`.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::net::SocketAddr;
use vectorvolt_cluster::shard::ShardTable;

#[derive(Args)]
pub struct ShardArgs {
    /// Control endpoint of a running cluster node.
    #[arg(long)]
    pub node: SocketAddr,

    #[command(subcommand)]
    pub command: ShardCommand,
}

#[derive(Subcommand)]
pub enum ShardCommand {
    /// List shards, owners, and in-flight migrations.
    List,
    /// Split a shard at KEY (default: its median key).
    Split {
        shard_id: u64,
        #[arg(long)]
        at: Option<u64>,
    },
    /// Live-migrate a shard to another node.
    Move {
        shard_id: u64,
        /// Target node id (uuid).
        to: String,
    },
    /// Migrate all shards away from a node, then mark it drained.
    Drain { node: String },
    /// Run one rebalance round (node-side rebalancer must be enabled).
    Rebalance,
}

fn http(node: SocketAddr, method: &str, path: &str, body: &str) -> Result<String> {
    use std::io::{Read, Write};
    let timeout = std::time::Duration::from_secs(120);
    let mut s = std::net::TcpStream::connect_timeout(&node, std::time::Duration::from_secs(10))
        .with_context(|| format!("cannot reach cluster node at {node}"))?;
    s.set_read_timeout(Some(timeout))?;
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {node}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes())?;
    let mut raw = String::new();
    s.read_to_string(&mut raw)?;
    let (head, resp) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    if code != 200 {
        anyhow::bail!("node returned {code}: {}", resp.trim());
    }
    Ok(resp.to_string())
}

fn render_table(t: &ShardTable) -> String {
    let mut out = format!("shard map version {}\n", t.version);
    out.push_str(&format!(
        "{:<8} {:<20} {:<20} {:<38} MIGRATING-TO\n",
        "SHARD", "LO", "HI", "OWNER"
    ));
    for s in t.shards.values() {
        let mig = t
            .migrating
            .get(&s.shard_id)
            .map(|n| n.0.to_string())
            .unwrap_or_default();
        out.push_str(&format!(
            "{:<8} {:<20} {:<20} {:<38} {}\n",
            s.shard_id, s.lo, s.hi, s.owner.0, mig
        ));
    }
    if !t.draining.is_empty() {
        let v: Vec<String> = t.draining.iter().map(|n| n.0.to_string()).collect();
        out.push_str(&format!("draining: {}\n", v.join(", ")));
    }
    if !t.drained.is_empty() {
        let v: Vec<String> = t.drained.iter().map(|n| n.0.to_string()).collect();
        out.push_str(&format!("drained:  {}\n", v.join(", ")));
    }
    out
}

/// Execute the command and return the text to print.
pub fn execute(args: &ShardArgs) -> Result<String> {
    let n = args.node;
    match &args.command {
        ShardCommand::List => {
            let t: ShardTable = serde_json::from_str(&http(n, "GET", "/shards", "")?)
                .context("node returned an invalid shard table")?;
            Ok(render_table(&t))
        }
        ShardCommand::Split { shard_id, at } => {
            let body = serde_json::json!({ "shard_id": shard_id, "at": at }).to_string();
            http(n, "POST", "/shards/split", &body)
        }
        ShardCommand::Move { shard_id, to } => {
            uuid::Uuid::parse_str(to).context("target must be a node uuid")?;
            let body = serde_json::json!({ "shard_id": shard_id, "to": to }).to_string();
            http(n, "POST", "/shards/move", &body)
        }
        ShardCommand::Drain { node } => {
            uuid::Uuid::parse_str(node).context("node must be a uuid")?;
            let body = serde_json::json!({ "node": node }).to_string();
            http(n, "POST", "/shards/drain", &body)
        }
        ShardCommand::Rebalance => http(n, "POST", "/shards/rebalance", "{}"),
    }
}

pub fn run(args: ShardArgs) -> Result<()> {
    println!("{}", execute(&args)?.trim_end());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use vectorvolt_cluster::node::NodeId;
    use vectorvolt_cluster::shard::ShardCmd;

    /// One-shot HTTP server; returns (addr, join handle yielding the request).
    fn serve_once(status: u16, body: String) -> (SocketAddr, std::thread::JoinHandle<String>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let h = std::thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp = format!(
                "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            s.write_all(resp.as_bytes()).unwrap();
            req
        });
        (addr, h)
    }

    #[test]
    fn list_renders_table() {
        let a = NodeId::new();
        let mut t = ShardTable::new();
        t.apply(&ShardCmd::Init { owner: a.clone() }).unwrap();
        t.apply(&ShardCmd::Split {
            shard_id: 1,
            at: 50,
        })
        .unwrap();
        let (addr, h) = serve_once(200, serde_json::to_string(&t).unwrap());
        let out = execute(&ShardArgs {
            node: addr,
            command: ShardCommand::List,
        })
        .unwrap();
        assert!(h.join().unwrap().starts_with("GET /shards "));
        assert!(out.contains("version 2"));
        assert!(out.contains(&a.0.to_string()));
        assert_eq!(
            out.lines().filter(|l| l.contains(&a.0.to_string())).count(),
            2
        );
    }

    #[test]
    fn split_and_move_post_json_and_errors_surface() {
        let (addr, h) = serve_once(200, "{\"ok\":true}".into());
        let out = execute(&ShardArgs {
            node: addr,
            command: ShardCommand::Split {
                shard_id: 3,
                at: Some(99),
            },
        })
        .unwrap();
        let req = h.join().unwrap();
        assert!(req.starts_with("POST /shards/split "));
        assert!(req.contains("\"shard_id\":3") && req.contains("\"at\":99"));
        assert!(out.contains("ok"));

        let (addr, h) = serve_once(409, "target is draining".into());
        let id = uuid::Uuid::new_v4().to_string();
        let err = execute(&ShardArgs {
            node: addr,
            command: ShardCommand::Move {
                shard_id: 1,
                to: id,
            },
        })
        .unwrap_err();
        h.join().unwrap();
        assert!(err.to_string().contains("409"));

        assert!(
            execute(&ShardArgs {
                node: addr,
                command: ShardCommand::Drain {
                    node: "nope".into()
                },
            })
            .is_err()
        );
    }
}
