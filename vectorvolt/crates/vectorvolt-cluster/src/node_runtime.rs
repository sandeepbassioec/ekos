//! Cluster node runtime wired into `vvctl serve`.
//!
//! A [`ClusterNode`] owns the real `MembershipList`. When
//! [`ClusterNode::start_gossip`] is called it binds a UDP socket on the
//! node's bind address and runs a SWIM [`SwimNode`] (probe / ping-req /
//! suspicion / refutation / piggy-backed dissemination), joining through the
//! configured `--cluster-seeds` with retry. Configured seeds only become
//! members once a seed actually answers.
//!
//! The control path used by `vvctl cluster ...` is the node's small HTTP
//! endpoint ([`ClusterStatusServer`]): `GET /cluster/status`,
//! `GET /cluster/members`, `POST /cluster/join?seed=host:port` and
//! `POST /cluster/leave` (mutations are accepted from loopback peers only).
use crate::gossip::GossipEngine;
use crate::gossip_net::UdpTransport;
use crate::membership::MembershipList;
use crate::node::{NodeId, NodeMeta, NodeState};
use crate::swim::{SwimConfig, SwimNode};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// A real cluster node; gossips over UDP once [`ClusterNode::start_gossip`] runs.
pub struct ClusterNode {
    node_id: NodeId,
    bind_addr: SocketAddr,
    configured_seeds: Vec<SocketAddr>,
    membership: Arc<MembershipList>,
    #[allow(dead_code)] // legacy in-memory delta engine, kept for API compat
    gossip: GossipEngine,
    swim: OnceLock<Arc<SwimNode>>,
    started_at: Instant,
}

impl ClusterNode {
    /// Construct the node (no I/O). `seeds` are recorded as configured
    /// addresses; `node_id` is generated if not supplied.
    pub fn start(
        bind_addr: SocketAddr,
        seeds: Vec<SocketAddr>,
        node_id: Option<NodeId>,
    ) -> Arc<Self> {
        let node_id = node_id.unwrap_or_default();
        let self_meta = NodeMeta {
            id: node_id.clone(),
            addr: bind_addr,
            state: NodeState::Alive,
            incarnation: 0,
            last_seen_ns: 0,
        };
        let membership = Arc::new(MembershipList::new(self_meta));
        let gossip = GossipEngine::new(membership.clone(), node_id.clone());
        Arc::new(Self {
            node_id,
            bind_addr,
            configured_seeds: seeds,
            membership,
            gossip,
            swim: OnceLock::new(),
            started_at: Instant::now(),
        })
    }

    /// Bind the UDP gossip socket on the bind address, start SWIM, and join
    /// through the configured seeds in the background (retrying until one
    /// answers or the node leaves).
    pub async fn start_gossip(self: &Arc<Self>, cfg: SwimConfig) -> std::io::Result<()> {
        let transport = UdpTransport::bind(self.bind_addr).await?;
        let swim = SwimNode::new(self.membership.clone(), transport, cfg);
        if self.swim.set(swim.clone()).is_err() {
            return Ok(());
        }
        swim.start();
        let seeds = self.configured_seeds.clone();
        if !seeds.is_empty() {
            tokio::spawn(async move {
                let mut delay = Duration::from_millis(200);
                while !swim.has_left() {
                    if swim.join(&seeds).await {
                        return;
                    }
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(Duration::from_secs(5));
                }
            });
        }
        Ok(())
    }

    /// Join a running cluster through `seed` (control-path entry point).
    pub async fn join_seed(&self, seed: SocketAddr) -> Result<(), String> {
        let swim = self
            .swim
            .get()
            .ok_or("gossip is not running on this node")?;
        if swim.join(&[seed]).await {
            Ok(())
        } else {
            Err(format!("seed {seed} did not answer"))
        }
    }

    /// Gracefully leave the cluster, broadcasting `Left`.
    pub async fn leave(&self) -> Result<(), String> {
        let swim = self
            .swim
            .get()
            .ok_or("gossip is not running on this node")?;
        swim.leave().await;
        Ok(())
    }

    /// Every member (including self) in any state.
    pub fn members(&self) -> Vec<PeerInfo> {
        let mut v: Vec<PeerInfo> = self
            .membership
            .all_members()
            .into_iter()
            .map(|m| PeerInfo {
                node_id: m.id.0.to_string(),
                addr: m.addr.to_string(),
                state: format!("{:?}", m.state),
                incarnation: m.incarnation,
            })
            .collect();
        v.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        v
    }

    pub fn status(&self) -> ClusterStatus {
        let me = self.node_id.0.to_string();
        let members = self.members();
        let count = |st: &str| members.iter().filter(|m| m.state == st).count();
        let known_peers: Vec<PeerInfo> = members
            .iter()
            .filter(|m| m.node_id != me)
            .cloned()
            .collect();
        let gossip_active = self.swim.get().is_some();
        ClusterStatus {
            node_id: me,
            bind_addr: self.bind_addr.to_string(),
            uptime_secs: self.started_at.elapsed().as_secs(),
            membership_size: self.membership.len(),
            alive: count("Alive"),
            suspect: count("Suspect"),
            dead: count("Dead"),
            left: count("Left"),
            gossip_active,
            known_peers,
            configured_seeds: self
                .configured_seeds
                .iter()
                .map(|a| a.to_string())
                .collect(),
            raft_leader: None,
            note: if gossip_active {
                "SWIM gossip over UDP active; configured_seeds become known_peers once a seed answers"
            } else {
                "gossip not started: configured_seeds are NOT reflected as known_peers"
            }
            .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub addr: String,
    pub state: String,
    #[serde(default)]
    pub incarnation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: String,
    pub bind_addr: String,
    pub uptime_secs: u64,
    pub membership_size: usize,
    #[serde(default)]
    pub alive: usize,
    #[serde(default)]
    pub suspect: usize,
    #[serde(default)]
    pub dead: usize,
    #[serde(default)]
    pub left: usize,
    #[serde(default)]
    pub gossip_active: bool,
    pub known_peers: Vec<PeerInfo>,
    pub configured_seeds: Vec<String>,
    pub raft_leader: Option<String>,
    pub note: String,
}

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string())
}

/// Minimal raw-TCP HTTP control endpoint (see module docs for routes).
pub struct ClusterStatusServer {
    node: Arc<ClusterNode>,
}

impl ClusterStatusServer {
    pub fn new(node: Arc<ClusterNode>) -> Self {
        Self { node }
    }

    /// Bind and serve forever on `port`.
    pub async fn serve(self, port: u16) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let addr = format!("0.0.0.0:{port}");
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("failed to bind cluster status endpoint on {addr}: {e}");
                return;
            }
        };

        loop {
            let (mut stream, peer) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("cluster status accept error: {e}");
                    continue;
                }
            };
            let node = self.node.clone();
            let loopback = peer.ip().is_loopback();
            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&buf[..n]);
                let line = head.lines().next().unwrap_or("");
                let mut parts = line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let target = parts.next().unwrap_or("");
                let (path, query) = target.split_once('?').unwrap_or((target, ""));
                let ok = "200 OK";
                let (status, content_type, body) = match (method, path) {
                    ("GET", "/cluster/status") => (ok, "application/json", json(&node.status())),
                    ("GET", "/cluster/members") => (ok, "application/json", json(&node.members())),
                    ("POST", "/cluster/join" | "/cluster/leave") if !loopback => (
                        "403 Forbidden",
                        "text/plain",
                        "mutations are accepted from loopback only\n".to_string(),
                    ),
                    ("POST", "/cluster/join") => {
                        let seed = query
                            .split('&')
                            .find_map(|kv| kv.strip_prefix("seed="))
                            .and_then(|v| v.parse::<SocketAddr>().ok());
                        match seed {
                            None => (
                                "400 Bad Request",
                                "text/plain",
                                "missing or invalid seed=host:port\n".to_string(),
                            ),
                            Some(seed) => match node.join_seed(seed).await {
                                Ok(()) => (ok, "application/json", json(&node.status())),
                                Err(e) => ("502 Bad Gateway", "text/plain", format!("{e}\n")),
                            },
                        }
                    }
                    ("POST", "/cluster/leave") => match node.leave().await {
                        Ok(()) => (ok, "application/json", json(&node.status())),
                        Err(e) => ("409 Conflict", "text/plain", format!("{e}\n")),
                    },
                    _ => ("404 Not Found", "text/plain", "404 Not Found\n".to_string()),
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    }
}
