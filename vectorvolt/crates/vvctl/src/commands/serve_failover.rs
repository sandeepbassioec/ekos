//! `vvctl serve` failover wiring (gap F6): the replication data plane driven
//! by the Raft control plane in `vectorvolt_cluster::failover`.
//!
//! [`FailoverNode::start`] runs the control plane for one table engine. The
//! Raft leader's [`ReplicationDriver::promote`] flips the node's [`RoleCell`]
//! to leader and starts a `ReplicationServer` (reachable only through the
//! epoch-fenced front); every other node's [`ReplicationDriver::follow`] makes
//! it read-only and catches up from the highest point it shares with the new
//! primary (an `(epoch, LSN)` history exchange): only its divergent un-acked
//! tail is rewritten and only the missing tail is streamed; a snapshot is the
//! fallback when no common point is known. The configured replication factor
//! is never lowered: with too few in-sync replicas a sync write is refused
//! ("insufficient replicas") unless `--replication-degraded-ok` is set, in
//! which case the node degrades explicitly (metric + log) and restores full
//! acknowledgement by itself when followers catch up. Write entry points (HTTP, gRPC, pgwire, native wire)
//! consult [`FailoverNode::write_gate`] before mutating and
//! [`FailoverNode::after_write`] after, which waits for replication in
//! sync / semi-sync modes.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use vectorvolt_cluster::failover::{FailoverConfig, FailoverController, Fence, RoleDriver};
use vectorvolt_cluster::node::NodeId;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::TableEngine;
use vectorvolt_replication::{
    EpochCell, Lineage, LineageStore, MetricsSnapshot, Position, ReplicationClient,
    ReplicationConfig, ReplicationRole, ReplicationServer, RoleCell, save_position_full,
};

/// A write-fencing / post-write check shared with every listener.
pub type Gate = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// Glue from gossip and `vvctl cluster join|leave` to the failover Raft
/// group's `add_server` / `remove_server`.
///
/// * `join` (control path): ask the group, through `seeds` (any member's
///   replication address; non-leaders forward to the leader), to add this node
///   (learner catch-up, then Raft promotes it to voter).
/// * `leave` (control path): ask the group to remove this node before the
///   gossip `Left` goes out.
/// * gossip snapshots: on the Raft leader, `Left` members are removed and
///   `Alive` members with known addresses are added
///   ([`FailoverController::reconcile_gossip`]).
pub struct RaftMembershipBridge {
    controller: Arc<FailoverController>,
    seeds: Vec<SocketAddr>,
}

impl RaftMembershipBridge {
    pub fn new(controller: Arc<FailoverController>, seeds: Vec<SocketAddr>) -> Self {
        Self { controller, seeds }
    }
}

#[async_trait]
impl vectorvolt_cluster::node_runtime::MembershipHook for RaftMembershipBridge {
    async fn joined(&self) {
        if !self.seeds.is_empty() {
            self.controller.auto_join(self.seeds.clone());
        }
    }

    async fn leaving(&self) {
        if self.seeds.is_empty() {
            return;
        }
        if let Err(e) = self.controller.leave(&self.seeds).await {
            eprintln!("failover: leaving the Raft group failed: {e}");
        }
    }

    async fn members(&self, members: Vec<vectorvolt_cluster::node_runtime::PeerInfo>) {
        let v: Vec<(String, String)> = members.into_iter().map(|m| (m.node_id, m.state)).collect();
        self.controller.reconcile_gossip(&v).await;
    }
}

/// Replication data plane for one table.
pub struct ReplicationDriver {
    engine: Arc<TableEngine>,
    dir: PathBuf,
    key: Option<Arc<KeyStore>>,
    cfg: ReplicationConfig,
    role: RoleCell,
    /// Highest primary epoch seen; followers drop anything older.
    known: EpochCell,
    /// Which primaries this node followed or led, for catch-up after failover.
    lineage: Arc<LineageStore>,
    slots: tokio::sync::Mutex<Slots>,
    /// Mirror of the live server for synchronous readers (write hooks).
    server: Mutex<Option<Arc<ReplicationServer>>>,
    /// Mirror of the live follower client (metrics).
    client: Mutex<Option<Arc<ReplicationClient>>>,
}

#[derive(Default)]
struct Slots {
    server: Option<Arc<ReplicationServer>>,
    client: Option<Arc<ReplicationClient>>,
}

impl ReplicationDriver {
    fn new(
        engine: Arc<TableEngine>,
        dir: PathBuf,
        key: Option<Arc<KeyStore>>,
        cfg: ReplicationConfig,
    ) -> Result<Self> {
        let lineage = Lineage::open(&dir).context("open replication lineage")?;
        Ok(ReplicationDriver {
            engine,
            dir,
            key,
            cfg,
            // Read-only until the control plane says otherwise.
            role: RoleCell::new(ReplicationRole::Follower),
            known: EpochCell::new(0),
            lineage: lineage.shared(),
            slots: tokio::sync::Mutex::new(Slots::default()),
            server: Mutex::new(None),
            client: Mutex::new(None),
        })
    }

    fn stop(&self, slots: &mut Slots) {
        if let Some(c) = slots.client.take() {
            c.shutdown();
        }
        if let Some(s) = slots.server.take() {
            s.shutdown();
        }
        *self.server.lock().expect("server") = None;
        *self.client.lock().expect("client") = None;
    }

    /// If this node led an epoch that has not been closed, close it at the
    /// current WAL tip and make that tip its follower position: the former
    /// primary then announces `(epoch, tip)` and the new primary can tell
    /// exactly which part of it was never replicated.
    fn close_primary_period(&self) {
        let tip = self.engine.wal_tip_sequence();
        let mut lin = self.lineage.lock();
        if let Some(e) = lin.open_primary_epoch()
            && let Err(err) = lin.end_primary(e, tip).and_then(|()| {
                save_position_full(
                    &self.dir,
                    &Position {
                        epoch: e,
                        lsn: tip,
                        primary: false,
                    },
                )
            })
        {
            eprintln!("failover: recording the end of primary epoch {e} failed: {err}");
        }
    }
}

#[async_trait]
impl RoleDriver for ReplicationDriver {
    async fn promote(&self, epoch: u64) -> Result<SocketAddr, String> {
        let mut slots = self.slots.lock().await;
        self.stop(&mut slots);
        self.known.advance(epoch);
        self.close_primary_period();
        let tip = self.engine.wal_tip_sequence();
        self.lineage
            .lock()
            .start_primary(epoch, tip)
            .map_err(|e| e.to_string())?;
        save_position_full(
            &self.dir,
            &Position {
                epoch,
                lsn: tip,
                primary: true,
            },
        )
        .map_err(|e| e.to_string())?;
        // Writable from here; the fence still gates client writes until the
        // controller opens it.
        self.role.set(ReplicationRole::Leader);
        let server = ReplicationServer::start_epoch(
            self.engine.clone(),
            self.dir.join("wal"),
            self.cfg.clone(),
            self.role.clone(),
            "127.0.0.1:0".parse().expect("addr"),
            epoch,
            self.lineage.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;
        let addr = server.local_addr();
        *self.server.lock().expect("server") = Some(server.clone());
        slots.server = Some(server);
        Ok(addr)
    }

    async fn demote(&self) {
        let mut slots = self.slots.lock().await;
        self.role.set(ReplicationRole::Follower);
        self.stop(&mut slots);
        self.close_primary_period();
    }

    async fn follow(&self, epoch: u64, leader_addr: SocketAddr) {
        let mut slots = self.slots.lock().await;
        self.role.set(ReplicationRole::Follower);
        self.stop(&mut slots);
        self.close_primary_period();
        self.known.advance(epoch);
        match ReplicationClient::start_epoch(
            self.engine.clone(),
            &self.dir,
            self.key.clone(),
            self.cfg.clone(),
            self.role.clone(),
            leader_addr.to_string(),
            self.known.clone(),
            self.lineage.clone(),
        ) {
            Ok(c) => {
                *self.client.lock().expect("client") = Some(c.clone());
                slots.client = Some(c);
            }
            Err(e) => eprintln!("failover: starting replication client failed: {e}"),
        }
    }
}

/// Everything needed to run one failover-enabled node.
pub struct FailoverNodeOpts {
    pub engine: Arc<TableEngine>,
    /// The table's data directory (WAL under `<dir>/wal`, Raft under `<dir>/failover`).
    pub dir: PathBuf,
    pub key: Option<Arc<KeyStore>>,
    pub repl: ReplicationConfig,
    pub control: FailoverConfig,
}

/// A running failover node: control plane + replication data plane.
pub struct FailoverNode {
    pub controller: Arc<FailoverController>,
    driver: Arc<ReplicationDriver>,
    engine: Arc<TableEngine>,
    monitor: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// Replication health of one node.
#[derive(Debug, Clone)]
pub struct ReplState {
    /// Followers caught up on this primary (0 on a follower).
    pub in_sync: usize,
    /// This primary currently acknowledges writes with fewer replicas than configured.
    pub degraded: bool,
    /// Primary: the server's counters; follower: the client's counters.
    pub metrics: Option<MetricsSnapshot>,
    /// Follower: the most recent replication session error.
    pub last_error: Option<String>,
}

impl FailoverNode {
    /// Start the control plane. Peers must be registered afterwards with
    /// [`FailoverController::add_peer`]; the node is read-only until elected
    /// primary or pointed at one.
    pub async fn start(opts: FailoverNodeOpts) -> Result<Arc<Self>> {
        let driver = Arc::new(ReplicationDriver::new(
            opts.engine.clone(),
            opts.dir.clone(),
            opts.key,
            opts.repl,
        )?);
        let controller = FailoverController::start(opts.control, driver.clone())
            .await
            .context("start failover control plane")?;
        // Say so, loudly and once per transition, when the primary runs below
        // the configured replica count.
        let watched = driver.clone();
        let monitor = tokio::spawn(async move {
            let mut degraded = false;
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let server = watched.server.lock().expect("server").clone();
                let now = server.as_ref().is_some_and(|s| s.is_degraded());
                if now != degraded {
                    degraded = now;
                    let (have, need) = server.as_ref().map_or((0, 0), |s| {
                        (s.in_sync_followers(), s.config().required_acks())
                    });
                    if now {
                        eprintln!(
                            "failover: replication DEGRADED: {have} of {need} required \
                             in-sync follower(s); writes are acknowledged with fewer replicas"
                        );
                    } else {
                        eprintln!("failover: replication restored to the configured factor");
                    }
                }
            }
        });
        Ok(Arc::new(FailoverNode {
            controller,
            driver,
            engine: opts.engine,
            monitor: Mutex::new(Some(monitor)),
        }))
    }

    pub fn fence(&self) -> Arc<Fence> {
        self.controller.fence()
    }

    pub fn role(&self) -> ReplicationRole {
        self.driver.role.get()
    }

    /// Replication health: in-sync follower count, degraded flag and counters.
    pub fn repl_state(&self) -> ReplState {
        if let Some(s) = self.driver.server.lock().expect("server").clone() {
            return ReplState {
                in_sync: s.in_sync_followers(),
                degraded: s.is_degraded(),
                metrics: Some(s.metrics().snapshot()),
                last_error: None,
            };
        }
        let client = self.driver.client.lock().expect("client").clone();
        ReplState {
            in_sync: 0,
            degraded: false,
            metrics: client.as_ref().map(|c| c.metrics().snapshot()),
            last_error: client.as_ref().and_then(|c| c.last_error()),
        }
    }

    /// Fail every mutating request with the current primary's address unless
    /// this node is the live primary for the current epoch.
    pub fn write_gate(&self) -> Gate {
        let fence = self.fence();
        let driver = self.driver.clone();
        Arc::new(move || {
            // The fence names the current primary in its refusal.
            fence.write_gate()?;
            // The replication role is the second, independent line of defence
            // (`ensure_writable`): a follower never writes even if the fence
            // were somehow open.
            driver.role.ensure_writable().map_err(|e| e.to_string())?;
            // Too few in-sync replicas for the configured mode (and degraded
            // writes not enabled): refuse *before* applying the write.
            let server = driver.server.lock().expect("server").clone();
            match server {
                Some(s) => s.check_ready().map_err(|e| e.to_string()),
                None => Ok(()),
            }
        })
    }

    /// Block until the most recent write is replicated as the configured mode
    /// requires (sync / semi-sync), then re-check that the node is still the
    /// primary for the epoch the write was accepted in. An `Err` means the
    /// write is committed locally but its replication is unconfirmed.
    pub fn after_write(&self) -> Gate {
        let fence = self.fence();
        let driver = self.driver.clone();
        let engine = self.engine.clone();
        Arc::new(move || {
            let epoch = fence.write_epoch();
            // Clone out first: holding the lock across the wait would serialise
            // every writer and block `kill`.
            let server = driver.server.lock().expect("server").clone();
            if let Some(server) = server {
                let lsn = engine.wal_tip_sequence();
                blocking(|| server.wait_for_replication_blocking(lsn))
                    .map_err(|e| e.to_string())?;
            }
            match (epoch, fence.check_write()) {
                (Some(a), Ok(b)) if a == b => Ok(()),
                (_, Err(e)) => Err(format!("primacy lost while replicating: {e}")),
                _ => Err("primary epoch changed while replicating; write unconfirmed".into()),
            }
        })
    }

    /// Insert through the full write path: fence, engine, replication wait.
    pub fn insert(&self, pk: u64, values: Vec<u64>) -> Result<(), String> {
        (self.write_gate())()?;
        self.engine.insert(pk, values).map_err(|e| e.to_string())?;
        (self.after_write())()
    }

    /// Stop the node abruptly (crash model): control plane and replication.
    pub fn kill(&self) {
        self.controller.crash();
        if let Some(m) = self.monitor.lock().expect("monitor").take() {
            m.abort();
        }
        if let Some(s) = self.driver.server.lock().expect("server").take() {
            s.shutdown();
        }
        if let Ok(mut slots) = self.driver.slots.try_lock() {
            self.driver.stop(&mut slots);
        }
        self.driver.role.set(ReplicationRole::Follower);
    }
}

/// Run a blocking wait from sync or async context.
fn blocking<T>(f: impl FnOnce() -> T) -> T {
    match tokio::runtime::Handle::try_current() {
        Ok(h) if h.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(f)
        }
        _ => f(),
    }
}

/// Parse `<uuid>` into a node id.
pub fn parse_node_id(s: &str) -> Result<NodeId, String> {
    uuid_parse(s)
        .map(NodeId)
        .ok_or_else(|| format!("invalid node id {s:?}"))
}

fn uuid_parse(s: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(s).ok()
}

/// A gate whose implementation is installed after the listener that holds it
/// was built (the net server is bound before the engine's failover node
/// exists). Until set it allows everything.
#[derive(Clone, Default)]
pub struct LateGate(Arc<std::sync::OnceLock<Gate>>);

impl LateGate {
    pub fn set(&self, g: Gate) {
        let _ = self.0.set(g);
    }
    pub fn is_set(&self) -> bool {
        self.0.get().is_some()
    }
    pub fn as_gate(&self) -> Gate {
        let cell = self.0.clone();
        Arc::new(move || cell.get().map_or(Ok(()), |g| g()))
    }
}

/// Parse a `--failover-peer` value: `<uuid>=<raft_addr>,<repl_addr>,<client_addr>`.
pub fn parse_peer(s: &str) -> Result<(NodeId, SocketAddr, SocketAddr, String), String> {
    let (id, rest) = s
        .split_once('=')
        .ok_or_else(|| format!("--failover-peer {s:?}: expected <uuid>=<raft>,<repl>,<client>"))?;
    let parts: Vec<&str> = rest.split(',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "--failover-peer {s:?}: expected <uuid>=<raft>,<repl>,<client>"
        ));
    }
    let addr = |p: &str| {
        p.parse::<SocketAddr>()
            .map_err(|e| format!("--failover-peer {s:?}: bad address {p:?}: {e}"))
    };
    Ok((
        parse_node_id(id)?,
        addr(parts[0])?,
        addr(parts[1])?,
        parts[2].to_string(),
    ))
}
