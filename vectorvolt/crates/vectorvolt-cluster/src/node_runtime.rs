//! Minimal real (non-simulated) cluster node runtime, wired for use from
//! `vvctl serve` (Gap #3).
//!
//! Scope note (see PR description for full rationale): `vectorvolt-cluster`
//! has no network transport at all today — `GossipEngine::make_gossip` /
//! `apply_gossip` are in-memory method calls, `SwimProber` has no real UDP
//! prober, and `raft::election::RaftCluster` is a deterministic in-process
//! step-simulation. There is no existing "dial a peer over a socket and
//! gossip with it" code anywhere in this crate. Building that transport from
//! scratch is out of scope for "wire the existing crate into serve".
//!
//! What this module DOES provide, and what `vvctl serve` now genuinely does:
//! - Constructs a real `NodeId`, real `MembershipList` (self-registered
//!   alive), and a real `GossipEngine` bound to that membership list — the
//!   same types exercised by `tests/s6_integration.rs`.
//! - Records `--cluster-seeds` as *configured* seed addresses. They are
//!   reported separately from `known_peers` and are NEVER upserted into the
//!   membership list as fake/hardcoded peers — until real gossip transport
//!   exists there is no way to honestly claim them as discovered members.
//! - Serves a minimal HTTP status endpoint (`GET /cluster/status`) so the
//!   node's real, live state (id, bind addr, uptime, membership list size,
//!   configured seeds) is observable from a running process, following the
//!   exact bare-bones raw-TCP pattern already used by
//!   `vectorvolt_metrics::MetricsServer`.
use crate::gossip::GossipEngine;
use crate::membership::MembershipList;
use crate::node::{NodeId, NodeMeta, NodeState};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

/// A real, running (but not yet network-connected) cluster node.
pub struct ClusterNode {
    node_id: NodeId,
    bind_addr: SocketAddr,
    configured_seeds: Vec<SocketAddr>,
    membership: Arc<MembershipList>,
    #[allow(dead_code)] // kept alive for future real gossip wiring
    gossip: GossipEngine,
    started_at: Instant,
}

impl ClusterNode {
    /// Start a real (single-process) cluster node bound to `bind_addr`.
    ///
    /// `seeds` are recorded as configured-but-unverified peer addresses.
    /// `node_id` is generated if not supplied.
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
            started_at: Instant::now(),
        })
    }

    pub fn status(&self) -> ClusterStatus {
        let known_peers: Vec<PeerInfo> = self
            .membership
            .alive_members()
            .into_iter()
            .filter(|m| m.id != self.node_id)
            .map(|m| PeerInfo {
                node_id: m.id.0.to_string(),
                addr: m.addr.to_string(),
                state: format!("{:?}", m.state),
            })
            .collect();
        ClusterStatus {
            node_id: self.node_id.0.to_string(),
            bind_addr: self.bind_addr.to_string(),
            uptime_secs: self.started_at.elapsed().as_secs(),
            membership_size: self.membership.len(),
            known_peers,
            configured_seeds: self
                .configured_seeds
                .iter()
                .map(|a| a.to_string())
                .collect(),
            raft_leader: None,
            note: "single-process node: gossip/Raft network transport is not \
                   implemented in vectorvolt-cluster yet, so configured_seeds \
                   are NOT reflected as known_peers until real wire-level \
                   gossip exists"
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub addr: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClusterStatus {
    pub node_id: String,
    pub bind_addr: String,
    pub uptime_secs: u64,
    pub membership_size: usize,
    pub known_peers: Vec<PeerInfo>,
    pub configured_seeds: Vec<String>,
    pub raft_leader: Option<String>,
    pub note: String,
}

/// Minimal raw-TCP HTTP server exposing `GET /cluster/status` as JSON.
/// Mirrors `vectorvolt_metrics::MetricsServer`'s bare-bones style.
pub struct ClusterStatusServer {
    node: Arc<ClusterNode>,
}

impl ClusterStatusServer {
    pub fn new(node: Arc<ClusterNode>) -> Self {
        Self { node }
    }

    /// Bind and serve forever on `port`. Intended to be spawned on its own
    /// tokio runtime, matching the other optional listeners in `serve.rs`.
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
            let (mut stream, _peer) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("cluster status accept error: {e}");
                    continue;
                }
            };

            let body = serde_json::to_string_pretty(&self.node.status())
                .unwrap_or_else(|_| "{}".to_string());
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf).await;

                let is_status = buf.starts_with(b"GET /cluster/status");
                let (status, content_type, body) = if is_status {
                    ("200 OK", "application/json", body)
                } else {
                    ("404 Not Found", "text/plain", "404 Not Found\n".to_string())
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
