//! Epoch-fenced failover (gap F6).
//!
//! A small Raft group across the cluster nodes is the *control plane*. Its
//! replicated state ([`ControlState`]) is `{epoch, primary node id, primary
//! term, per-node replication/client addresses}`. The Raft leader is the write
//! primary for the data:
//!
//! 1. A node that wins an election proposes `Promote { epoch + 1 }`
//!    (compare-and-set on the epoch, so concurrent promotions cannot both
//!    land). The committed epoch is the fencing token.
//! 2. The supervisor then drives a [`RoleDriver`] (the data plane): the new
//!    primary starts replication serving, every other node re-points its
//!    replication client at the primary and rebuilds from its snapshot.
//! 3. Writes are gated by a [`Fence`]: only a node that is the Raft leader, has
//!    had its own promotion applied, and has committed a heartbeat entry within
//!    the last `lease` accepts writes. A partitioned or deposed primary
//!    therefore stops accepting writes within `lease`, which is shorter than the
//!    minimum election timeout, before any new primary can start writing.
//! 4. Replication streams are epoch-scoped: the follower announces its epoch in
//!    a handshake ([`ClientTunnel`]) and the primary's replication front
//!    ([`ReplFront`]) rejects any stream from another epoch (or when it is not
//!    the primary for its own epoch), and drops live streams when its epoch
//!    changes.
//!
//! 5. Membership is dynamic: the nodes named at start are only the bootstrap
//!    set. A node joins or leaves the group at runtime through the Raft
//!    membership API (`add_server` / `promote_server` / `remove_server`),
//!    requested over the replication port ([`request_membership`]) and recorded
//!    in the control state so every node learns the newcomer's addresses.
//!
//! Failover time is the election timeout (`election_ticks * tick` to twice
//! that) plus one Raft commit round plus driver start-up.

use crate::node::NodeId;
use crate::raft::node::{RaftConfig, RaftNode, StateMachine};
use crate::raft::server::{self, RaftHandle};
use crate::raft::state::RaftRole;
use crate::transport::{TcpConfig, TcpTransport};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

// ---- replicated control state ----------------------------------------------

/// Addresses a node advertises through the control plane.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAddrs {
    /// Raft RPC endpoint.
    #[serde(default)]
    pub raft: String,
    /// Fenced replication endpoint ([`ReplFront`]).
    pub repl: String,
    /// Client endpoint named in "not primary" errors.
    pub client: String,
}

/// The Raft-replicated control state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlState {
    /// Fencing epoch; bumped by every committed promotion.
    pub epoch: u64,
    /// Node id string of the current primary.
    pub primary: Option<String>,
    /// Raft term in which `primary` was promoted.
    pub primary_term: u64,
    /// Addresses of every member: announced by the promoting leader and kept
    /// current by `SetNode` / `RemoveNode`.
    pub nodes: BTreeMap<String, NodeAddrs>,
    /// Members that left the group (so every node forgets their addresses).
    #[serde(default)]
    pub removed: BTreeSet<String>,
    /// Log index of the latest `SetNode` per member: a node re-reads a
    /// member's addresses only when this moves past what it already applied.
    #[serde(default)]
    pub node_rev: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Cmd {
    Promote {
        node: String,
        epoch: u64,
        term: u64,
        nodes: BTreeMap<String, NodeAddrs>,
    },
    /// Lease heartbeat: carries no state, its commit proves quorum contact.
    Touch,
    /// A member joined (or changed its addresses).
    SetNode { id: String, addrs: NodeAddrs },
    /// A member left the group.
    RemoveNode { id: String },
}

struct ControlSm {
    shared: Arc<RwLock<ControlState>>,
}

impl StateMachine for ControlSm {
    fn apply(&mut self, index: u64, command: &[u8]) {
        let Ok(cmd) = serde_json::from_slice::<Cmd>(command) else {
            return;
        };
        let mut st = self.shared.write().expect("control state");
        match cmd {
            Cmd::Promote {
                node,
                epoch,
                term,
                mut nodes,
            } => {
                // Compare-and-set: only the next epoch may be installed.
                if epoch == st.epoch + 1 {
                    nodes.retain(|k, _| !st.removed.contains(k));
                    st.epoch = epoch;
                    st.primary = Some(node);
                    st.primary_term = term;
                    st.nodes = nodes;
                }
            }
            Cmd::SetNode { id, addrs } => {
                st.removed.remove(&id);
                st.node_rev.insert(id.clone(), index);
                st.nodes.insert(id, addrs);
            }
            Cmd::RemoveNode { id } => {
                st.nodes.remove(&id);
                st.removed.insert(id);
            }
            Cmd::Touch => {}
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        serde_json::to_vec(&*self.shared.read().expect("control state")).unwrap_or_default()
    }

    fn restore(&mut self, data: &[u8]) {
        if let Ok(st) = serde_json::from_slice::<ControlState>(data) {
            *self.shared.write().expect("control state") = st;
        }
    }
}

// ---- write fence -----------------------------------------------------------

/// Why a write was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FenceError {
    #[error("not the primary for epoch {epoch}; current primary: {primary}")]
    NotPrimary { epoch: u64, primary: String },
    #[error("no primary available at epoch {epoch}; retry shortly")]
    NoPrimary { epoch: u64 },
    #[error("primary lease expired at epoch {epoch}; this node stepped down")]
    LeaseExpired { epoch: u64 },
}

#[derive(Default)]
struct PrimaryInfo {
    node: Option<String>,
    client: String,
}

/// Cheap, lock-light write gate shared with every write entry point.
pub struct Fence {
    me: String,
    start: Instant,
    /// Latest committed epoch this node has seen.
    epoch: AtomicU64,
    /// Epoch for which this node may accept writes (0 = none).
    write_epoch: AtomicU64,
    /// Milliseconds since `start` until which the primary lease holds.
    lease_until_ms: AtomicU64,
    primary: RwLock<PrimaryInfo>,
}

impl Fence {
    fn new(me: String) -> Self {
        Fence {
            me,
            start: Instant::now(),
            epoch: AtomicU64::new(0),
            write_epoch: AtomicU64::new(0),
            lease_until_ms: AtomicU64::new(0),
            primary: RwLock::new(PrimaryInfo::default()),
        }
    }

    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Latest committed epoch known locally.
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    /// Epoch this node currently accepts writes for, if it is the primary.
    pub fn write_epoch(&self) -> Option<u64> {
        match self.write_epoch.load(Ordering::Acquire) {
            0 => None,
            e => Some(e),
        }
    }

    /// Address of the current primary as announced through the control plane.
    pub fn primary_client_addr(&self) -> Option<String> {
        let p = self.primary.read().expect("primary");
        (!p.client.is_empty()).then(|| p.client.clone())
    }

    /// `Ok(epoch)` when this node may accept a write now.
    pub fn check_write(&self) -> Result<u64, FenceError> {
        let epoch = self.epoch();
        let we = self.write_epoch.load(Ordering::Acquire);
        if we == 0 {
            return Err(self.refusal(epoch));
        }
        if we != epoch {
            return Err(self.refusal(epoch));
        }
        if self.now_ms() > self.lease_until_ms.load(Ordering::Acquire) {
            return Err(FenceError::LeaseExpired { epoch });
        }
        Ok(epoch)
    }

    /// True while writes are accepted.
    pub fn is_primary(&self) -> bool {
        self.check_write().is_ok()
    }

    /// Error text suitable for a client: names the current primary.
    pub fn write_gate(&self) -> Result<(), String> {
        self.check_write().map(|_| ()).map_err(|e| e.to_string())
    }

    fn refusal(&self, epoch: u64) -> FenceError {
        let p = self.primary.read().expect("primary");
        match &p.node {
            Some(n) if *n != self.me && !p.client.is_empty() => FenceError::NotPrimary {
                epoch,
                primary: p.client.clone(),
            },
            _ => FenceError::NoPrimary { epoch },
        }
    }

    fn publish(&self, st: &ControlState) {
        self.epoch.fetch_max(st.epoch, Ordering::AcqRel);
        let mut p = self.primary.write().expect("primary");
        p.node = st.primary.clone();
        p.client = st
            .primary
            .as_ref()
            .and_then(|n| st.nodes.get(n))
            .map(|a| a.client.clone())
            .unwrap_or_default();
    }

    fn open(&self, epoch: u64, lease_until_ms: u64) {
        self.lease_until_ms.store(lease_until_ms, Ordering::Release);
        self.write_epoch.store(epoch, Ordering::Release);
    }

    fn close(&self) {
        self.write_epoch.store(0, Ordering::Release);
    }
}

// ---- epoch-scoped replication streams --------------------------------------

const HS_MAGIC: &[u8; 4] = b"VFE1";
const MBR_MAGIC: &[u8; 4] = b"VFJ1";

/// Handshake verdicts returned by [`ReplFront`].
pub mod hs {
    pub const OK: u8 = 0;
    /// The connecting follower's epoch is older than the primary's.
    pub const STALE_CLIENT: u8 = 1;
    /// The follower's epoch is newer than anything this node knows of.
    pub const STALE_SERVER: u8 = 2;
    /// This node is not the (live) primary for its epoch.
    pub const NOT_PRIMARY: u8 = 3;
}

/// Server's answer to an epoch handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeReply {
    pub status: u8,
    pub server_epoch: u64,
}

impl HandshakeReply {
    pub fn accepted(&self) -> bool {
        self.status == hs::OK
    }
}

async fn send_handshake(s: &mut TcpStream, epoch: u64, node: &str) -> io::Result<HandshakeReply> {
    let id = node.as_bytes();
    let mut buf = Vec::with_capacity(13 + id.len());
    buf.extend_from_slice(HS_MAGIC);
    buf.extend_from_slice(&epoch.to_le_bytes());
    buf.push(id.len().min(255) as u8);
    buf.extend_from_slice(&id[..id.len().min(255)]);
    s.write_all(&buf).await?;
    let mut r = [0u8; 9];
    s.read_exact(&mut r).await?;
    Ok(HandshakeReply {
        status: r[0],
        server_epoch: u64::from_le_bytes(r[1..9].try_into().expect("8 bytes")),
    })
}

/// Open a connection to `addr`, present `epoch`, and return the verdict
/// (the connection is dropped). Used by tests and diagnostics.
pub async fn probe_repl_epoch(addr: SocketAddr, epoch: u64) -> io::Result<HandshakeReply> {
    let mut s = tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr))
        .await
        .map_err(|_| io::ErrorKind::TimedOut)??;
    tokio::time::timeout(
        Duration::from_secs(2),
        send_handshake(&mut s, epoch, "probe"),
    )
    .await
    .map_err(|_| io::ErrorKind::TimedOut)?
}

/// A request to join or leave the failover group, sent to any member's
/// replication port (non-leaders answer with the leader's address).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipRequest {
    /// `"join"` or `"leave"`.
    pub op: String,
    /// Shared secret; required when the receiving node has an admin token,
    /// otherwise only loopback callers are accepted.
    pub token: Option<String>,
    pub id: String,
    pub raft: String,
    pub repl: String,
    pub client: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MembershipReply {
    pub ok: bool,
    pub error: String,
    /// Replication address of the current Raft leader when this node is not it.
    pub leader_repl: Option<String>,
}

/// Why a membership change was not made.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MembershipError {
    #[error("not the Raft leader (leader: {leader_repl:?})")]
    NotLeader { leader_repl: Option<String> },
    #[error("membership change refused: {0}")]
    Refused(String),
}

type MemberSlot = Arc<OnceLock<Weak<Inner>>>;

async fn read_json<T: serde::de::DeserializeOwned>(s: &mut TcpStream) -> io::Result<T> {
    let mut len = [0u8; 4];
    s.read_exact(&mut len).await?;
    let n = u32::from_le_bytes(len) as usize;
    if n > 64 * 1024 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut buf = vec![0u8; n];
    s.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf).map_err(io::Error::other)
}

async fn write_json<T: Serialize>(s: &mut TcpStream, v: &T) -> io::Result<()> {
    let body = serde_json::to_vec(v).map_err(io::Error::other)?;
    s.write_all(&(body.len() as u32).to_le_bytes()).await?;
    s.write_all(&body).await?;
    s.flush().await
}

/// Ask the group (via any member's replication address, following leader
/// redirects) to apply `req`.
pub async fn request_membership(
    seeds: &[SocketAddr],
    req: &MembershipRequest,
) -> Result<(), String> {
    let mut queue: Vec<SocketAddr> = seeds.to_vec();
    let mut last = "no seed addresses".to_string();
    let mut hops = 0;
    while let Some(addr) = (!queue.is_empty()).then(|| queue.remove(0)) {
        hops += 1;
        if hops > 12 {
            break;
        }
        let attempt = async {
            let mut s = tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr))
                .await
                .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
            s.write_all(MBR_MAGIC).await?;
            write_json(&mut s, req).await?;
            tokio::time::timeout(
                Duration::from_secs(10),
                read_json::<MembershipReply>(&mut s),
            )
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
        };
        match attempt.await {
            Ok(r) if r.ok => return Ok(()),
            Ok(r) => {
                last = format!("{addr}: {}", r.error);
                if let Some(l) = r.leader_repl.and_then(|l| l.parse::<SocketAddr>().ok()) {
                    queue.insert(0, l);
                }
            }
            Err(e) => last = format!("{addr}: {e}"),
        }
    }
    Err(last)
}

/// The primary's public, fenced replication endpoint. Accepts a stream only
/// after a successful epoch handshake, then forwards it to the local
/// replication server; streams are cut when the node's write epoch changes.
pub struct ReplFront {
    local_addr: SocketAddr,
    backend: Arc<Mutex<Option<SocketAddr>>>,
    task: JoinHandle<()>,
}

impl ReplFront {
    async fn start(listen: SocketAddr, fence: Arc<Fence>, members: MemberSlot) -> io::Result<Self> {
        let listener = TcpListener::bind(listen).await?;
        let local_addr = listener.local_addr()?;
        let backend: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
        let b = backend.clone();
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    continue;
                };
                let backend = b.clone();
                let fence = fence.clone();
                let members = members.clone();
                tokio::spawn(async move {
                    let _ = serve_front_conn(&mut stream, backend, fence, members).await;
                });
            }
        });
        Ok(ReplFront {
            local_addr,
            backend,
            task,
        })
    }

    fn set_backend(&self, addr: Option<SocketAddr>) {
        *self.backend.lock().expect("backend") = addr;
    }
}

async fn serve_front_conn(
    client: &mut TcpStream,
    backend: Arc<Mutex<Option<SocketAddr>>>,
    fence: Arc<Fence>,
    members: MemberSlot,
) -> io::Result<()> {
    let mut magic = [0u8; 4];
    tokio::time::timeout(Duration::from_secs(2), client.read_exact(&mut magic))
        .await
        .map_err(|_| io::ErrorKind::TimedOut)??;
    if &magic == MBR_MAGIC {
        let loopback = client.peer_addr().is_ok_and(|a| a.ip().is_loopback());
        let req: MembershipRequest = read_json(client).await?;
        let reply = match members.get().and_then(Weak::upgrade) {
            Some(inner) => inner.handle_membership(req, loopback).await,
            None => MembershipReply {
                ok: false,
                error: "node is starting".into(),
                leader_repl: None,
            },
        };
        return write_json(client, &reply).await;
    }
    if &magic != HS_MAGIC {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut head = [0u8; 9];
    tokio::time::timeout(Duration::from_secs(2), client.read_exact(&mut head))
        .await
        .map_err(|_| io::ErrorKind::TimedOut)??;
    let client_epoch = u64::from_le_bytes(head[..8].try_into().expect("8 bytes"));
    let mut id = vec![0u8; head[8] as usize];
    client.read_exact(&mut id).await?;

    let server_epoch = fence.epoch();
    let write_epoch = fence.write_epoch.load(Ordering::Acquire);
    let target = *backend.lock().expect("backend");
    let status = if client_epoch < server_epoch {
        hs::STALE_CLIENT
    } else if client_epoch > server_epoch {
        hs::STALE_SERVER
    } else if write_epoch != client_epoch || target.is_none() {
        hs::NOT_PRIMARY
    } else {
        hs::OK
    };
    let mut reply = [0u8; 9];
    reply[0] = status;
    reply[1..9].copy_from_slice(&server_epoch.to_le_bytes());
    client.write_all(&reply).await?;
    client.flush().await?;
    if status != hs::OK {
        return Ok(());
    }
    let mut upstream = TcpStream::connect(target.expect("checked")).await?;
    let _ = upstream.set_nodelay(true);
    let _ = client.set_nodelay(true);
    let watch = async {
        loop {
            tokio::time::sleep(Duration::from_millis(25)).await;
            if fence.write_epoch.load(Ordering::Acquire) != client_epoch {
                return;
            }
        }
    };
    tokio::select! {
        _ = tokio::io::copy_bidirectional(client, &mut upstream) => {}
        _ = watch => {}
    }
    Ok(())
}

/// Follower-side counterpart of [`ReplFront`]: a loopback listener that dials
/// the primary, presents this node's epoch and, only if accepted, pipes the
/// replication stream. Point the replication client at [`local_addr`].
///
/// [`local_addr`]: ClientTunnel::local_addr
pub struct ClientTunnel {
    local_addr: SocketAddr,
    task: JoinHandle<()>,
}

impl ClientTunnel {
    pub async fn start(target: SocketAddr, epoch: u64, node: String) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = listener.local_addr()?;
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut local, _)) = listener.accept().await else {
                    continue;
                };
                let node = node.clone();
                tokio::spawn(async move {
                    let Ok(Ok(mut up)) =
                        tokio::time::timeout(Duration::from_secs(1), TcpStream::connect(target))
                            .await
                    else {
                        return;
                    };
                    let Ok(Ok(reply)) = tokio::time::timeout(
                        Duration::from_secs(2),
                        send_handshake(&mut up, epoch, &node),
                    )
                    .await
                    else {
                        return;
                    };
                    if !reply.accepted() {
                        return; // dropping `local` makes the client back off and retry
                    }
                    let _ = local.set_nodelay(true);
                    let _ = tokio::io::copy_bidirectional(&mut local, &mut up).await;
                });
            }
        });
        Ok(ClientTunnel { local_addr, task })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for ClientTunnel {
    fn drop(&mut self) {
        self.task.abort();
    }
}

// ---- data-plane driver -----------------------------------------------------

/// The data plane the supervisor drives. Implementations must be idempotent.
#[async_trait]
pub trait RoleDriver: Send + Sync + 'static {
    /// Become the write primary for `epoch`: stop following, switch to leader,
    /// start replication serving. Returns the local address of the replication
    /// server; the controller exposes it through the fenced [`ReplFront`].
    ///
    /// The configured replication factor is kept: with too few replicas alive
    /// the driver either rejects sync writes ("insufficient replicas") or, when
    /// degraded writes are enabled, degrades explicitly — it never silently
    /// lowers the factor.
    async fn promote(&self, epoch: u64) -> Result<SocketAddr, String>;
    /// Stop acting as primary or follower; the node becomes read-only and idle.
    async fn demote(&self);
    /// Follow the primary of `epoch` through `leader_addr` (a local fenced
    /// tunnel). The node must refuse writes and catch up from a point it shares
    /// with the new primary, dropping only its own divergent tail.
    async fn follow(&self, epoch: u64, leader_addr: SocketAddr);
}

// ---- controller ------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FailoverConfig {
    pub node_id: NodeId,
    /// Directory for the Raft log / hard state.
    pub dir: PathBuf,
    /// Raft RPC listener (TCP; port 0 allowed).
    pub raft_listen: SocketAddr,
    /// Every other control-plane member's id (static membership).
    pub peer_ids: Vec<NodeId>,
    /// Public fenced replication endpoint (port 0 allowed).
    pub repl_listen: SocketAddr,
    /// Client endpoint advertised in "not primary" errors.
    pub client_addr: String,
    /// Open as a *joining* node: not a voter until the leader has added it
    /// (see [`FailoverController::auto_join`]). `peer_ids` then only names
    /// the members to recognise.
    pub join: bool,
    /// Shared secret for membership requests over the replication port; without
    /// one only loopback callers may change membership.
    pub admin_token: Option<String>,
    /// Wall-clock length of one Raft tick.
    pub tick: Duration,
    /// Election timeout is uniform in `[election_ticks, 2*election_ticks)` ticks.
    pub election_ticks: u64,
    pub heartbeat_ticks: u64,
    /// Primary write lease; must stay below `election_ticks * tick`.
    pub lease: Duration,
    /// Supervisor poll period.
    pub poll: Duration,
    pub seed: u64,
}

impl FailoverConfig {
    pub fn new(node_id: NodeId, dir: PathBuf, peer_ids: Vec<NodeId>) -> Self {
        FailoverConfig {
            node_id,
            dir,
            raft_listen: "127.0.0.1:0".parse().expect("addr"),
            peer_ids,
            repl_listen: "127.0.0.1:0".parse().expect("addr"),
            client_addr: String::new(),
            join: false,
            admin_token: None,
            tick: Duration::from_millis(20),
            election_ticks: 8,
            heartbeat_ticks: 2,
            lease: Duration::from_millis(120),
            poll: Duration::from_millis(15),
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DriverState {
    Idle,
    Primary { epoch: u64, term: u64 },
    Follower { epoch: u64, primary: String },
}

struct Inner {
    me: String,
    cfg: FailoverConfig,
    fence: Arc<Fence>,
    shared: Arc<RwLock<ControlState>>,
    transport: Arc<TcpTransport>,
    front: ReplFront,
    /// Every known node's addresses (self included), announced on promotion.
    known: Mutex<BTreeMap<String, NodeAddrs>>,
    raft_handle: RaftHandle,
}

/// Runs the control-plane Raft group and the failover supervisor.
pub struct FailoverController {
    inner: Arc<Inner>,
    supervisor: Mutex<Option<JoinHandle<()>>>,
    joiner: Mutex<Option<JoinHandle<()>>>,
}

impl FailoverController {
    /// Start Raft, the fenced replication front and the supervisor.
    pub async fn start(cfg: FailoverConfig, driver: Arc<dyn RoleDriver>) -> io::Result<Arc<Self>> {
        std::fs::create_dir_all(&cfg.dir)?;
        let me = cfg.node_id.0.to_string();
        let shared = Arc::new(RwLock::new(ControlState::default()));
        let transport = Arc::new(
            TcpTransport::bind(
                cfg.node_id.clone(),
                cfg.raft_listen,
                HashMap::new(),
                TcpConfig::default(),
            )
            .await?,
        );
        let raft_cfg = RaftConfig {
            election_ticks: cfg.election_ticks,
            heartbeat_ticks: cfg.heartbeat_ticks,
            snapshot_threshold: 512,
            seed: cfg.seed ^ cfg.node_id.0.as_u128() as u64,
            ..RaftConfig::default()
        };
        let sm = ControlSm {
            shared: shared.clone(),
        };
        let (id, peers, dir) = (
            cfg.node_id.clone(),
            cfg.peer_ids.clone(),
            cfg.dir.join("raft"),
        );
        let node = if cfg.join {
            RaftNode::open_joining(id, peers, raft_cfg, dir, sm)
        } else {
            RaftNode::open(id, peers, raft_cfg, dir, sm)
        }
        .map_err(|e| io::Error::other(e.to_string()))?;
        let raft_handle = server::spawn(node, transport.clone(), cfg.tick);
        let fence = Arc::new(Fence::new(me.clone()));
        let member_slot: MemberSlot = Arc::new(OnceLock::new());
        let front = ReplFront::start(cfg.repl_listen, fence.clone(), member_slot.clone()).await?;
        let mut known = BTreeMap::new();
        known.insert(
            me.clone(),
            NodeAddrs {
                raft: transport.local_addr().to_string(),
                repl: front.local_addr.to_string(),
                client: cfg.client_addr.clone(),
            },
        );
        let inner = Arc::new(Inner {
            me,
            cfg,
            fence,
            shared,
            transport,
            front,
            known: Mutex::new(known),
            raft_handle,
        });
        let _ = member_slot.set(Arc::downgrade(&inner));
        let sup = tokio::spawn(supervise(inner.clone(), driver));
        Ok(Arc::new(FailoverController {
            inner,
            supervisor: Mutex::new(Some(sup)),
            joiner: Mutex::new(None),
        }))
    }

    /// Address of the Raft RPC listener.
    pub fn raft_addr(&self) -> SocketAddr {
        self.inner.transport.local_addr()
    }

    /// Address of the fenced replication endpoint.
    pub fn repl_addr(&self) -> SocketAddr {
        self.inner.front.local_addr
    }

    pub fn node_id(&self) -> &str {
        &self.inner.me
    }

    /// Register a peer: its Raft address and the addresses it advertises.
    pub fn add_peer(&self, id: &NodeId, raft: SocketAddr, repl: SocketAddr, client: String) {
        self.inner.transport.set_peer(id.clone(), raft);
        self.inner.known.lock().expect("known").insert(
            id.0.to_string(),
            NodeAddrs {
                raft: raft.to_string(),
                repl: repl.to_string(),
                client,
            },
        );
    }

    /// This node's own advertised addresses.
    pub fn my_addrs(&self) -> NodeAddrs {
        self.inner
            .known
            .lock()
            .expect("known")
            .get(&self.inner.me)
            .cloned()
            .unwrap_or_default()
    }

    /// Raft status as this node sees it (role, leader, voters, learners).
    pub async fn raft_status(&self) -> Option<crate::raft::node::Status> {
        self.inner.raft_handle.status().await
    }

    /// Leader-only: add `id` to the group (non-voting learner first; the
    /// leader promotes it to voter once it has caught up).
    pub async fn join_member(&self, id: &NodeId, addrs: NodeAddrs) -> Result<(), MembershipError> {
        self.inner.join_member(id.clone(), addrs).await
    }

    /// Reconcile the Raft group with gossip membership (`(node id, SWIM state)`
    /// pairs, state as `Alive` / `Left` / ...). Acts only on the Raft leader:
    /// a gossip-`Left` member still in the group is removed; an `Alive` member
    /// whose addresses are known, that is not in the group and was not
    /// explicitly removed, is added (learner first, then promoted by Raft).
    /// Returns how many changes were made.
    pub async fn reconcile_gossip(&self, members: &[(String, String)]) -> usize {
        let Some(st) = self.inner.raft_handle.status().await else {
            return 0;
        };
        if st.role != RaftRole::Leader {
            return 0;
        }
        let ctl = self.state();
        let mut changed = 0;
        for (sid, state) in members {
            let Ok(u) = uuid::Uuid::parse_str(sid) else {
                continue;
            };
            let id = NodeId(u);
            if *sid == self.inner.me {
                continue;
            }
            let in_group = st.voters.contains(&id) || st.learners.contains(&id);
            let r = match state.as_str() {
                "Left" if in_group => self.inner.leave_member(id).await,
                "Alive" if !in_group && !ctl.removed.contains(sid) => {
                    let addrs = self.inner.known.lock().expect("known").get(sid).cloned();
                    match addrs {
                        Some(a) if !a.raft.is_empty() => self.inner.join_member(id, a).await,
                        _ => continue,
                    }
                }
                _ => continue,
            };
            match r {
                Ok(()) => changed += 1,
                Err(e) => eprintln!("failover: gossip membership change for {sid}: {e}"),
            }
        }
        changed
    }

    /// Leader-only: remove `id` from the group.
    pub async fn leave_member(&self, id: &NodeId) -> Result<(), MembershipError> {
        self.inner.leave_member(id.clone()).await
    }

    fn membership_request(&self, op: &str) -> MembershipRequest {
        let a = self.my_addrs();
        MembershipRequest {
            op: op.into(),
            token: self.inner.cfg.admin_token.clone(),
            id: self.inner.me.clone(),
            raft: a.raft,
            repl: a.repl,
            client: a.client,
        }
    }

    /// Keep asking `seeds` (any members' replication addresses) to add this
    /// node until one does. Runs in the background.
    pub fn auto_join(&self, seeds: Vec<SocketAddr>) {
        let req = self.membership_request("join");
        let task = tokio::spawn(async move {
            for _ in 0..240 {
                if request_membership(&seeds, &req).await.is_ok() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            eprintln!("failover: could not join the group through {seeds:?}");
        });
        if let Some(old) = self.joiner.lock().expect("joiner").replace(task) {
            old.abort();
        }
    }

    /// Ask the group (through `seeds`) to remove this node, then stop it.
    pub async fn leave(&self, seeds: &[SocketAddr]) -> Result<(), String> {
        let r = request_membership(seeds, &self.membership_request("leave")).await;
        self.crash();
        r
    }

    pub fn fence(&self) -> Arc<Fence> {
        self.inner.fence.clone()
    }

    /// Snapshot of the replicated control state as applied locally.
    pub fn state(&self) -> ControlState {
        self.inner.shared.read().expect("control state").clone()
    }

    /// Stop the control plane abruptly (models a crash of the control process).
    pub fn crash(&self) {
        if let Some(h) = self.joiner.lock().expect("joiner").take() {
            h.abort();
        }
        if let Some(h) = self.supervisor.lock().expect("supervisor").take() {
            h.abort();
        }
        self.inner.fence.close();
        self.inner.raft_handle.crash();
        self.inner.front.task.abort();
    }
}

impl Inner {
    fn leader_repl(&self, status: &crate::raft::node::Status) -> Option<String> {
        let l = status.leader_id.as_ref()?;
        self.known
            .lock()
            .expect("known")
            .get(&l.0.to_string())
            .map(|a| a.repl.clone())
            .filter(|r| !r.is_empty())
    }

    async fn leader_status(&self) -> Result<crate::raft::node::Status, MembershipError> {
        let status = self
            .raft_handle
            .status()
            .await
            .ok_or_else(|| MembershipError::Refused("control plane stopped".into()))?;
        if status.role != RaftRole::Leader {
            return Err(MembershipError::NotLeader {
                leader_repl: self.leader_repl(&status),
            });
        }
        Ok(status)
    }

    async fn join_member(&self, id: NodeId, addrs: NodeAddrs) -> Result<(), MembershipError> {
        let status = self.leader_status().await?;
        let raft: SocketAddr = addrs
            .raft
            .parse()
            .map_err(|e| MembershipError::Refused(format!("bad raft address: {e}")))?;
        self.transport.set_peer(id.clone(), raft);
        let sid = id.0.to_string();
        self.known
            .lock()
            .expect("known")
            .insert(sid.clone(), addrs.clone());
        let cmd = serde_json::to_vec(&Cmd::SetNode { id: sid, addrs }).unwrap_or_default();
        self.raft_handle
            .propose(cmd)
            .await
            .map_err(|e| MembershipError::Refused(e.to_string()))?;
        if !status.voters.contains(&id) && !status.learners.contains(&id) {
            self.raft_handle
                .add_server(id)
                .await
                .map_err(|e| MembershipError::Refused(e.to_string()))?;
        }
        Ok(())
    }

    async fn leave_member(&self, id: NodeId) -> Result<(), MembershipError> {
        let status = self.leader_status().await?;
        let cmd = serde_json::to_vec(&Cmd::RemoveNode {
            id: id.0.to_string(),
        })
        .unwrap_or_default();
        self.raft_handle
            .propose(cmd)
            .await
            .map_err(|e| MembershipError::Refused(e.to_string()))?;
        if status.voters.contains(&id) || status.learners.contains(&id) {
            self.raft_handle
                .remove_server(id)
                .await
                .map_err(|e| MembershipError::Refused(e.to_string()))?;
        }
        Ok(())
    }

    async fn handle_membership(&self, req: MembershipRequest, loopback: bool) -> MembershipReply {
        let refuse = |error: String| MembershipReply {
            ok: false,
            error,
            leader_repl: None,
        };
        let authorised = match &self.cfg.admin_token {
            Some(t) => req.token.as_deref() == Some(t.as_str()),
            None => loopback,
        };
        if !authorised {
            return refuse("membership changes need the admin token".into());
        }
        let Ok(uuid) = uuid::Uuid::parse_str(&req.id) else {
            return refuse(format!("invalid node id {:?}", req.id));
        };
        let id = NodeId(uuid);
        let result = match req.op.as_str() {
            "join" => {
                self.join_member(
                    id,
                    NodeAddrs {
                        raft: req.raft,
                        repl: req.repl,
                        client: req.client,
                    },
                )
                .await
            }
            "leave" => self.leave_member(id).await,
            other => Err(MembershipError::Refused(format!("unknown op {other:?}"))),
        };
        match result {
            Ok(()) => MembershipReply {
                ok: true,
                ..MembershipReply::default()
            },
            Err(MembershipError::NotLeader { leader_repl }) => MembershipReply {
                ok: false,
                error: "not the Raft leader".into(),
                leader_repl,
            },
            Err(e) => refuse(e.to_string()),
        }
    }
}

/// Learn newly announced members' addresses (and forget departed ones).
/// Addresses wired locally win over the ones recorded at promotion time (a
/// restarted peer has new sockets); a member's explicit `SetNode` wins once.
fn sync_members(inner: &Inner, st: &ControlState, synced: &mut HashMap<String, u64>) {
    let mut known = inner.known.lock().expect("known");
    for (id, a) in &st.nodes {
        if *id == inner.me {
            continue;
        }
        let rev = st.node_rev.get(id).copied().unwrap_or(0);
        let fresh = rev > synced.get(id).copied().unwrap_or(0);
        if known.contains_key(id) && !fresh {
            continue;
        }
        synced.insert(id.clone(), rev);
        known.insert(id.clone(), a.clone());
        if let (Ok(u), Ok(addr)) = (uuid::Uuid::parse_str(id), a.raft.parse::<SocketAddr>()) {
            inner.transport.set_peer(NodeId(u), addr);
        }
    }
    for id in &st.removed {
        if *id != inner.me
            && known.remove(id).is_some()
            && let Ok(u) = uuid::Uuid::parse_str(id)
        {
            inner.transport.remove_peer(&NodeId(u));
        }
    }
}

impl Drop for FailoverController {
    fn drop(&mut self) {
        self.crash();
    }
}

#[allow(clippy::too_many_lines)]
async fn supervise(inner: Arc<Inner>, driver: Arc<dyn RoleDriver>) {
    let cfg = &inner.cfg;
    let mut ds = DriverState::Idle;
    let mut tunnel: Option<ClientTunnel> = None;
    // (log index, term, sent) of the outstanding lease heartbeat.
    let mut touch: Option<(u64, u64, Instant)> = None;
    let mut promote_sent: Option<(u64, Instant)> = None;
    let mut lease_until: Option<Instant> = None;
    let mut last_touch = Instant::now() - Duration::from_secs(1);
    let mut synced: HashMap<String, u64> = HashMap::new();

    loop {
        tokio::time::sleep(cfg.poll).await;
        let st = inner.shared.read().expect("control state").clone();
        inner.fence.publish(&st);
        sync_members(&inner, &st, &mut synced);
        let Some(status) = inner.raft_handle.status().await else {
            inner.fence.close();
            return;
        };
        let leader = status.role == RaftRole::Leader;

        // ---- leader duties: promotion + lease heartbeat ----
        let promoted = leader
            && st.primary.as_deref() == Some(inner.me.as_str())
            && st.primary_term == status.term;
        if leader {
            if let Some((idx, term, sent)) = touch {
                if term != status.term || sent.elapsed() > Duration::from_secs(1) {
                    touch = None;
                } else if status.commit_index >= idx {
                    lease_until = Some(sent + cfg.lease);
                    touch = None;
                }
            }
            if !promoted
                && promote_sent.is_none_or(|(t, at)| {
                    t != status.term || at.elapsed() > Duration::from_millis(300)
                })
            {
                let cmd = Cmd::Promote {
                    node: inner.me.clone(),
                    epoch: st.epoch + 1,
                    term: status.term,
                    nodes: inner.known.lock().expect("known").clone(),
                };
                if inner
                    .raft_handle
                    .propose(serde_json::to_vec(&cmd).unwrap_or_default())
                    .await
                    .is_ok()
                {
                    promote_sent = Some((status.term, Instant::now()));
                }
            }
            if touch.is_none() && last_touch.elapsed() >= cfg.lease / 4 {
                let sent = Instant::now();
                last_touch = sent;
                if let Ok(idx) = inner
                    .raft_handle
                    .propose(serde_json::to_vec(&Cmd::Touch).unwrap_or_default())
                    .await
                {
                    touch = Some((idx, status.term, sent));
                }
            }
        } else {
            touch = None;
            promote_sent = None;
            lease_until = None;
        }
        let lease_alive = lease_until.is_some_and(|t| Instant::now() < t);

        // ---- close the write fence the moment primacy is in doubt ----
        let may_write = promoted && lease_alive;
        if !may_write {
            inner.fence.close();
        }

        // ---- desired data-plane state ----
        let want = if st.removed.contains(&inner.me) {
            DriverState::Idle
        } else if promoted {
            DriverState::Primary {
                epoch: st.epoch,
                term: status.term,
            }
        } else if !leader
            && let Some(p) = st.primary.clone()
            && p != inner.me
            && st.nodes.contains_key(&p)
        {
            DriverState::Follower {
                epoch: st.epoch,
                primary: p,
            }
        } else {
            DriverState::Idle
        };

        if want != ds {
            // Leaving any state: drop the stream tunnel; leaving Primary also
            // withdraws the replication backend.
            let was_primary = matches!(ds, DriverState::Primary { .. });
            tunnel = None;
            if was_primary || matches!(want, DriverState::Idle) {
                inner.fence.close();
                inner.front.set_backend(None);
            }
            match &want {
                DriverState::Idle => {
                    driver.demote().await;
                    ds = DriverState::Idle;
                }
                DriverState::Primary { epoch, .. } => {
                    inner.fence.close();
                    inner.front.set_backend(None);
                    match driver.promote(*epoch).await {
                        Ok(addr) => {
                            inner.front.set_backend(Some(addr));
                            ds = want.clone();
                        }
                        Err(e) => {
                            eprintln!("failover: promote(epoch {epoch}) failed: {e}");
                            ds = DriverState::Idle;
                        }
                    }
                }
                DriverState::Follower { epoch, primary } => {
                    let target = st
                        .nodes
                        .get(primary)
                        .and_then(|a| a.repl.parse::<SocketAddr>().ok());
                    match target {
                        Some(t) => match ClientTunnel::start(t, *epoch, inner.me.clone()).await {
                            Ok(tun) => {
                                let local = tun.local_addr();
                                tunnel = Some(tun);
                                driver.follow(*epoch, local).await;
                                ds = want.clone();
                            }
                            Err(e) => eprintln!("failover: tunnel to {t} failed: {e}"),
                        },
                        None => eprintln!("failover: primary {primary} has no replication address"),
                    }
                }
            }
        }

        // ---- open the fence only for a live, fully promoted primary ----
        if may_write
            && let DriverState::Primary { epoch, .. } = ds
            && epoch == st.epoch
            && let Some(l) = lease_until
        {
            let ms = inner.fence.now_ms()
                + l.saturating_duration_since(Instant::now()).as_millis() as u64;
            inner.fence.open(epoch, ms);
        }
        let _ = &tunnel;
    }
}
