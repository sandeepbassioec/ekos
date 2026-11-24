//! Production wiring of the cluster features into `vvctl serve` (gap F7/F8).
//!
//! One call from `serve::run` ([`start`]) does the following:
//!
//! * **F8 versionstamps**: one [`VersionstampClock`] per node, persisted at
//!   `<dir>/versionstamp.clock`, installed as the engine's commit stamper
//!   (`TableEngine::set_commit_stamper`). Every committed change carries its
//!   stamp in the CDC event (and thus the CDC log, SSE and sinks). The clock
//!   epoch follows the failover fencing epoch once [`set_epoch_source`] is
//!   given one (else 0). `GET /versionstamp` returns a fresh stamp.
//! * **F7 shards on the real engine**: [`EngineShardNode`] is the shard data
//!   plane (rows of the node's `TableEngine`; copy through
//!   `TableEngine::consistent_snapshot` plus the live change stream; freeze
//!   and cutover through the engine's write guard). A Raft group of the nodes
//!   in `<dir>/shard_cluster.json` (single node when absent) commits the
//!   shard map. `/shards*` (admin, what `vvctl shard` talks to) and
//!   `/shardnode/*` (node-to-node data plane) are mounted on the node's
//!   control HTTP server (`--cluster-status-port`).
//! * **F8 sagas**: `POST /sagas`, `GET /sagas/{id}`; every step runs as one
//!   atomic batch on the node owning the step's shard (locally or via
//!   `/shardnode/exec`), the saga log lives in `<dir>/sagas`, and unfinished
//!   sagas resume at startup.
//!
//! `<dir>/shard_cluster.json`:
//! `{"raft_listen":"ip:port"?, "peers":[{"id":"<uuid>","raft":"ip:port","http":"ip:port"}]}`
//! lists the *other* nodes (raft RPC address and control-HTTP address). The
//! Raft listener defaults to `--cluster-bind` (TCP; gossip uses UDP on the same
//! port). Mutations and the data plane need the shared `--secret` (sent as
//! `x-vv-secret`) or a loopback peer.

use super::serve::ServeArgs;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;
use vectorvolt_cluster::migration::{
    Change, DataError, MapControl, MigrationConfig, MigrationJournal, Nodes, RaftMapControl,
    RaftMember, Recovery, ShardAdmin, ShardLoad, ShardNode, bootstrap, recover_migrations,
};
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::node_runtime::{ExtHandler, ExtRequest, install_ext};
use vectorvolt_cluster::raft::node::{RaftConfig, RaftNode};
use vectorvolt_cluster::raft::server::{self, RaftHandle};
use vectorvolt_cluster::raft::state::RaftRole;
use vectorvolt_cluster::rebalance::{RebalanceConfig, Rebalancer};
use vectorvolt_cluster::saga::{
    Record, SagaCoordinator, SagaDef, SagaLog, SagaOutcome, SagaStepExecutor, StepDef, StepError,
    StepRequest,
};
use vectorvolt_cluster::shard::{ShardMapSm, SharedTable, median_split_key};
use vectorvolt_cluster::shard_rpc::{RemoteShardNode, http_call, serve_shard_rpc};
use vectorvolt_cluster::transport::{TcpConfig, TcpTransport};
use vectorvolt_cluster::versionstamp::{Versionstamp, VersionstampClock, system_wall_clock};
use vectorvolt_engine::TableEngine;
use vectorvolt_engine::cdc::{ChangeEvent, ChangeKind};

// ---------------------------------------------------------------------------
// process-wide pieces
// ---------------------------------------------------------------------------

static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static EPOCH_SRC: OnceLock<Arc<dyn Fn() -> u64 + Send + Sync>> = OnceLock::new();
static CLOCK: OnceLock<Arc<VersionstampClock>> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("cluster-ext")
            .enable_all()
            .build()
            .expect("cluster-ext runtime")
    })
}

/// Give the versionstamp clock its epoch: the failover fencing epoch. The
/// closure runs once per commit, so it must be cheap (e.g. an atomic load such
/// as `fence.epoch()`). Returns `false` if a source is already set.
pub fn set_epoch_source(f: Arc<dyn Fn() -> u64 + Send + Sync>) -> bool {
    EPOCH_SRC.set(f).is_ok()
}

/// The node's versionstamp clock (after [`start`] ran with a single-table engine).
pub fn clock() -> Option<Arc<VersionstampClock>> {
    CLOCK.get().cloned()
}

/// Merge a stamp received from another node (replication apply, saga step,
/// migration stream): the clock advances so the next local stamp is greater.
pub fn observe_remote_stamp(bytes: &[u8]) -> bool {
    match (CLOCK.get(), Versionstamp::from_bytes(bytes)) {
        (Some(c), Some(s)) => c.observe(s).is_ok(),
        _ => false,
    }
}

fn hash16(id: &NodeId) -> u16 {
    // FNV-1a over the uuid: stable per node, any good mix works.
    let b = id.0.as_bytes();
    let mut h = 0x811c_9dc5u32;
    for x in b {
        h ^= *x as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    (h & 0xffff) as u16
}

// ---------------------------------------------------------------------------
// engine-backed shard data plane
// ---------------------------------------------------------------------------

thread_local! {
    /// Set while the shard plane itself writes (import, stream apply, release)
    /// so its writes bypass the ownership/freeze guard.
    static BYPASS: Cell<bool> = const { Cell::new(false) };
}

fn bypass<T>(f: impl FnOnce() -> T) -> T {
    BYPASS.with(|b| b.set(true));
    let r = f();
    BYPASS.with(|b| b.set(false));
    r
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    Serving,
    Frozen,
    Importing,
}

struct Capture {
    rx: tokio::sync::broadcast::Receiver<ChangeEvent>,
    /// Commit timestamp of the consistent snapshot.
    snap_ts: u64,
    /// Keys in range at the snapshot, ascending. Rows are *not* held: each
    /// `scan` chunk reads its rows from the engine as of `snap_ts`, so the
    /// buffered row data is bounded by the chunk size.
    keys: Vec<u64>,
    log: Vec<Change>,
    seq: u64,
}

struct Hosted {
    id: u64,
    lo: u64,
    hi: u64,
    st: St,
    ops: AtomicU64,
    cap: Mutex<Option<Capture>>,
}

impl Hosted {
    fn contains(&self, k: u64) -> bool {
        k >= self.lo && (k < self.hi || self.hi == u64::MAX)
    }
}

struct Inner {
    /// Keyed by `lo` (ranges are disjoint).
    shards: RwLock<BTreeMap<u64, Hosted>>,
    /// Becomes true once the node hosts its first shard; before that the guard
    /// admits everything (a node that is not part of any shard map).
    active: AtomicBool,
}

impl Inner {
    fn guard(&self, pk: u64) -> Result<(), String> {
        if !self.active.load(Ordering::Acquire) || BYPASS.with(|b| b.get()) {
            return Ok(());
        }
        let g = self.shards.read().unwrap();
        match g
            .range(..=pk)
            .next_back()
            .map(|(_, h)| h)
            .filter(|h| h.contains(pk))
        {
            Some(h) => match h.st {
                St::Serving => {
                    h.ops.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                St::Frozen => Err("shard:frozen".into()),
                St::Importing => Err("shard:not_owner".into()),
            },
            None => Err("shard:not_owner".into()),
        }
    }
}

fn enc(values: &[u64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn dec(b: &[u8]) -> Result<Vec<u64>, DataError> {
    let (chunks, rest) = b.as_chunks::<8>();
    if !rest.is_empty() {
        return Err(DataError::Other("row value must be 8*N bytes".into()));
    }
    Ok(chunks.iter().map(|c| u64::from_le_bytes(*c)).collect())
}

fn map_err(e: impl std::fmt::Display) -> DataError {
    let s = e.to_string();
    if s.contains("shard:frozen") {
        DataError::Frozen
    } else if s.contains("shard:not_owner") {
        DataError::NotOwner
    } else {
        DataError::Other(s)
    }
}

/// [`ShardNode`] over the node's real [`TableEngine`]: a shard is a key range
/// of the table; values are the row's columns as little-endian `u64`s.
pub struct EngineShardNode {
    id: NodeId,
    engine: Arc<TableEngine>,
    inner: Arc<Inner>,
    /// Largest number of rows buffered by one migration snapshot chunk.
    peak_scan_rows: AtomicU64,
}

impl EngineShardNode {
    /// Create the node and install its ownership/freeze guard on the engine.
    pub fn new(id: NodeId, engine: Arc<TableEngine>) -> Arc<Self> {
        let inner = Arc::new(Inner {
            shards: RwLock::new(BTreeMap::new()),
            active: AtomicBool::new(false),
        });
        let i2 = Arc::clone(&inner);
        engine.set_write_guard(Some(Arc::new(move |pk| i2.guard(pk))));
        Arc::new(EngineShardNode {
            id,
            engine,
            inner,
            peak_scan_rows: AtomicU64::new(0),
        })
    }

    /// Peak rows held in memory by a migration snapshot chunk (not the whole
    /// shard: the snapshot streams in `limit`-sized chunks).
    pub fn peak_scan_rows(&self) -> u64 {
        self.peak_scan_rows.load(Ordering::Relaxed)
    }
    /// Start enforcing ownership now (instead of at the first hosted shard): used
    /// when the node is configured with shard peers, so a node that owns nothing
    /// rejects writes to keys it does not own.
    pub fn enforce_ownership(&self) {
        self.inner.active.store(true, Ordering::Release);
    }

    pub fn is_hosted(&self, shard_id: u64) -> bool {
        self.inner
            .shards
            .read()
            .unwrap()
            .values()
            .any(|h| h.id == shard_id)
    }

    fn range_of(&self, shard_id: u64) -> Result<(u64, u64), DataError> {
        self.inner
            .shards
            .read()
            .unwrap()
            .values()
            .find(|h| h.id == shard_id)
            .map(|h| (h.lo, h.hi))
            .ok_or(DataError::NotHosted(shard_id))
    }

    /// Ascending keys of `[lo, hi)` at a consistent snapshot, with its timestamp.
    fn keys_in(&self, lo: u64, hi: u64) -> (u64, Vec<u64>) {
        let (ts, keys) = self.engine.consistent_snapshot_keys();
        let mut v: Vec<u64> = keys
            .into_iter()
            .filter(|k| *k >= lo && (*k < hi || hi == u64::MAX))
            .collect();
        v.sort_unstable();
        (ts, v)
    }

    fn apply_ops(&self, ops: Vec<(u64, Option<Vec<u64>>)>) -> Result<(), DataError> {
        if ops.is_empty() {
            return Ok(());
        }
        bypass(|| self.engine.apply_batch(ops))
            .map(|_| ())
            .map_err(map_err)
    }

    async fn delete_range(&self, lo: u64, hi: u64) -> Result<(), DataError> {
        let eng = Arc::clone(&self.engine);
        let rows = tokio::task::spawn_blocking(move || {
            let (_, keys) = eng.consistent_snapshot_keys();
            keys.into_iter()
                .filter(|k| *k >= lo && (*k < hi || hi == u64::MAX))
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|e| DataError::Other(e.to_string()))?;
        for chunk in rows.chunks(512) {
            self.apply_ops(chunk.iter().map(|k| (*k, None)).collect())?;
        }
        Ok(())
    }

    /// Run one saga step's operations on this node as a single atomic batch.
    pub fn exec_local(&self, shard_id: u64, ops: &[SagaOp]) -> Result<(), StepError> {
        let (lo, hi) = {
            let g = self.inner.shards.read().unwrap();
            let h = g
                .values()
                .find(|h| h.id == shard_id)
                .ok_or_else(|| StepError::Transient(format!("shard {shard_id} not hosted here")))?;
            match h.st {
                St::Serving => {}
                St::Frozen => {
                    return Err(StepError::Transient(format!("shard {shard_id} is frozen")));
                }
                St::Importing => {
                    return Err(StepError::Transient(format!(
                        "shard {shard_id} not serving"
                    )));
                }
            }
            (h.lo, h.hi)
        };
        let in_range = |k: u64| k >= lo && (k < hi || hi == u64::MAX);
        let mut batch: Vec<(u64, Option<Vec<u64>>)> = Vec::new();
        for op in ops {
            let key = op.key();
            if !in_range(key) {
                return Err(StepError::Permanent(format!(
                    "key {key} is outside shard {shard_id} [{lo},{hi})"
                )));
            }
            match op {
                SagaOp::Expect {
                    key,
                    values,
                    absent,
                } => {
                    let cur = self
                        .engine
                        .get(*key)
                        .map_err(|e| StepError::Transient(e.to_string()))?;
                    let ok = match (absent, values, &cur) {
                        (true, _, None) => true,
                        (true, _, Some(_)) => false,
                        (false, None, Some(_)) => true,
                        (false, Some(v), Some(r)) => &r.values == v,
                        (false, _, None) => false,
                    };
                    if !ok {
                        return Err(StepError::Permanent(format!(
                            "expectation failed on key {key} (found {:?})",
                            cur.map(|r| r.values)
                        )));
                    }
                }
                SagaOp::Put { key, values } => batch.push((*key, Some(values.clone()))),
                SagaOp::Delete { key } => batch.push((*key, None)),
            }
        }
        if batch.is_empty() {
            return Ok(());
        }
        // Not bypassed: the engine guard enforces ownership/freeze atomically.
        self.engine.apply_batch(batch).map(|_| ()).map_err(|e| {
            let s = e.to_string();
            if s.contains("shard:") {
                StepError::Transient(s)
            } else {
                StepError::Permanent(s)
            }
        })
    }
}

#[async_trait]
impl ShardNode for EngineShardNode {
    fn id(&self) -> NodeId {
        self.id.clone()
    }

    async fn host_shard(
        &self,
        shard_id: u64,
        lo: u64,
        hi: u64,
        serving: bool,
    ) -> Result<(), DataError> {
        let mut g = self.inner.shards.write().unwrap();
        if g.values()
            .any(|h| h.id == shard_id && h.st != St::Importing)
        {
            return Err(DataError::Other(format!("shard {shard_id} already hosted")));
        }
        g.retain(|_, h| h.id != shard_id);
        g.insert(
            lo,
            Hosted {
                id: shard_id,
                lo,
                hi,
                st: if serving { St::Serving } else { St::Importing },
                ops: AtomicU64::new(0),
                cap: Mutex::new(None),
            },
        );
        self.inner.active.store(true, Ordering::Release);
        Ok(())
    }

    async fn put(&self, key: u64, value: Vec<u8>) -> Result<(), DataError> {
        let vals = dec(&value)?;
        self.engine.insert(key, vals).map(|_| ()).map_err(map_err)
    }

    async fn delete(&self, key: u64) -> Result<(), DataError> {
        self.engine.delete(key).map(|_| ()).map_err(map_err)
    }

    async fn get(&self, key: u64) -> Result<Option<Vec<u8>>, DataError> {
        if self.inner.active.load(Ordering::Acquire) {
            let g = self.inner.shards.read().unwrap();
            let owned = g
                .range(..=key)
                .next_back()
                .map(|(_, h)| h)
                .is_some_and(|h| h.contains(key) && h.st != St::Importing);
            if !owned {
                return Err(DataError::NotOwner);
            }
        }
        Ok(self
            .engine
            .get(key)
            .map_err(map_err)?
            .map(|r| enc(&r.values)))
    }

    async fn begin_capture(&self, shard_id: u64) -> Result<u64, DataError> {
        let (lo, hi) = self.range_of(shard_id)?;
        // Subscribe first: every change after this point is streamed, the
        // snapshot then covers everything before it (duplicates converge).
        let rx = self.engine.subscribe_changes();
        let eng = Arc::clone(&self.engine);
        let (snap_ts, keys) = tokio::task::spawn_blocking(move || {
            let (ts, keys) = eng.consistent_snapshot_keys();
            let mut v: Vec<u64> = keys
                .into_iter()
                .filter(|k| *k >= lo && (*k < hi || hi == u64::MAX))
                .collect();
            v.sort_unstable();
            (ts, v)
        })
        .await
        .map_err(|e| DataError::Other(e.to_string()))?;
        let g = self.inner.shards.read().unwrap();
        let h = g
            .values()
            .find(|h| h.id == shard_id)
            .ok_or(DataError::NotHosted(shard_id))?;
        *h.cap.lock().unwrap() = Some(Capture {
            rx,
            snap_ts,
            keys,
            log: Vec::new(),
            seq: 0,
        });
        Ok(0)
    }

    async fn scan(
        &self,
        shard_id: u64,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<(u64, Vec<u8>)>, DataError> {
        let g = self.inner.shards.read().unwrap();
        let h = g
            .values()
            .find(|h| h.id == shard_id)
            .ok_or(DataError::NotHosted(shard_id))?;
        let cap = h.cap.lock().unwrap();
        let cap = cap
            .as_ref()
            .ok_or_else(|| DataError::Other("no capture in progress".into()))?;
        let start = match after {
            Some(a) => cap.keys.partition_point(|k| *k <= a),
            None => 0,
        };
        // One bounded chunk, read as of the snapshot. A key whose row is gone
        // at that timestamp is skipped (the change stream carries its delete).
        let mut out = Vec::with_capacity(limit.min(cap.keys.len() - start));
        for k in &cap.keys[start..] {
            if out.len() >= limit {
                break;
            }
            if let Some(r) = self.engine.get_at(*k, cap.snap_ts) {
                out.push((*k, enc(&r.values)));
            }
        }
        self.peak_scan_rows
            .fetch_max(out.len() as u64, Ordering::Relaxed);
        Ok(out)
    }

    async fn changes_since(
        &self,
        shard_id: u64,
        seq: u64,
        limit: usize,
    ) -> Result<Vec<Change>, DataError> {
        use tokio::sync::broadcast::error::TryRecvError;
        let g = self.inner.shards.read().unwrap();
        let h = g
            .values()
            .find(|h| h.id == shard_id)
            .ok_or(DataError::NotHosted(shard_id))?;
        let mut cap = h.cap.lock().unwrap();
        let cap = cap
            .as_mut()
            .ok_or_else(|| DataError::Other("no capture in progress".into()))?;
        loop {
            match cap.rx.try_recv() {
                Ok(ev) => {
                    if !h.contains(ev.pk) {
                        continue;
                    }
                    cap.seq += 1;
                    let value = match ev.kind {
                        ChangeKind::Insert => ev.values.as_deref().map(enc),
                        ChangeKind::Delete => None,
                    };
                    cap.log.push(Change {
                        seq: cap.seq,
                        key: ev.pk,
                        value,
                    });
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Closed) => break,
                Err(TryRecvError::Lagged(n)) => {
                    return Err(DataError::Other(format!(
                        "change capture lagged by {n} events; retry the migration"
                    )));
                }
            }
        }
        cap.log.retain(|c| c.seq > seq);
        Ok(cap.log.iter().take(limit).cloned().collect())
    }

    async fn apply_import(
        &self,
        shard_id: u64,
        entries: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), DataError> {
        self.range_of(shard_id)?;
        let mut ops = Vec::with_capacity(entries.len());
        for (k, v) in entries {
            ops.push((k, Some(dec(&v)?)));
        }
        self.apply_ops(ops)
    }

    async fn apply_changes(&self, shard_id: u64, changes: Vec<Change>) -> Result<(), DataError> {
        self.range_of(shard_id)?;
        // Last write per key wins; the stream is already in commit order.
        let mut last: BTreeMap<u64, Option<Vec<u64>>> = BTreeMap::new();
        for c in changes {
            let v = match c.value {
                Some(b) => Some(dec(&b)?),
                None => None,
            };
            last.insert(c.key, v);
        }
        self.apply_ops(last.into_iter().collect())
    }

    async fn freeze(&self, shard_id: u64) -> Result<(), DataError> {
        {
            let mut g = self.inner.shards.write().unwrap();
            let h = g
                .values_mut()
                .find(|h| h.id == shard_id)
                .ok_or(DataError::NotHosted(shard_id))?;
            h.st = St::Frozen;
        }
        // Wait for writes that passed the guard before it flipped.
        if !self.engine.quiesce_writes(Duration::from_secs(10)) {
            return Err(DataError::Other(
                "timed out draining in-flight writes".into(),
            ));
        }
        Ok(())
    }

    async fn rollback_source(&self, shard_id: u64) -> Result<(), DataError> {
        let mut g = self.inner.shards.write().unwrap();
        if let Some(h) = g.values_mut().find(|h| h.id == shard_id) {
            if h.st == St::Frozen {
                h.st = St::Serving;
            }
            *h.cap.lock().unwrap() = None;
        }
        Ok(())
    }

    async fn activate(&self, shard_id: u64) -> Result<(), DataError> {
        let mut g = self.inner.shards.write().unwrap();
        let h = g
            .values_mut()
            .find(|h| h.id == shard_id)
            .ok_or(DataError::NotHosted(shard_id))?;
        h.st = St::Serving;
        *h.cap.lock().unwrap() = None;
        Ok(())
    }

    async fn release(&self, shard_id: u64) -> Result<(), DataError> {
        let removed = {
            let mut g = self.inner.shards.write().unwrap();
            let lo = g.values().find(|h| h.id == shard_id).map(|h| (h.lo, h.hi));
            if let Some((lo, _)) = lo {
                g.remove(&lo);
            }
            lo
        };
        match removed {
            Some((lo, hi)) => self.delete_range(lo, hi).await,
            None => Ok(()),
        }
    }

    async fn discard(&self, shard_id: u64) -> Result<(), DataError> {
        // Same effect as release: drop the (partial) import and its rows.
        self.release(shard_id).await
    }

    async fn split_local(&self, shard_id: u64, at: u64, new_id: u64) -> Result<(), DataError> {
        let mut g = self.inner.shards.write().unwrap();
        let h = g
            .values_mut()
            .find(|h| h.id == shard_id)
            .ok_or(DataError::NotHosted(shard_id))?;
        if at <= h.lo || (at >= h.hi && h.hi != u64::MAX) {
            return Err(DataError::Other("split point outside shard".into()));
        }
        let (hi, st) = (h.hi, h.st);
        h.hi = at;
        g.insert(
            at,
            Hosted {
                id: new_id,
                lo: at,
                hi,
                st,
                ops: AtomicU64::new(0),
                cap: Mutex::new(None),
            },
        );
        Ok(())
    }

    async fn median_key(&self, shard_id: u64) -> Result<Option<u64>, DataError> {
        let (lo, hi) = self.range_of(shard_id)?;
        let (_, keys) = self.keys_in(lo, hi);
        Ok(median_split_key(&keys, lo))
    }

    async fn loads(&self) -> Result<Vec<ShardLoad>, DataError> {
        let spans: Vec<(u64, u64, u64, u64)> = self
            .inner
            .shards
            .read()
            .unwrap()
            .values()
            .map(|h| (h.id, h.lo, h.hi, h.ops.load(Ordering::Relaxed)))
            .collect();
        let mut out = Vec::new();
        for (id, lo, hi, ops) in spans {
            out.push(ShardLoad {
                shard_id: id,
                keys: self.keys_in(lo, hi).1.len() as u64,
                ops_total: ops,
            });
        }
        out.sort_by_key(|l| l.shard_id);
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// sagas on the real tables
// ---------------------------------------------------------------------------

/// One operation of a saga step, applied atomically with its siblings on the
/// node owning the step's shard. `put`/`delete` are absolute (idempotent);
/// `expect` fails the step permanently when the row is not as stated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SagaOp {
    Put {
        key: u64,
        values: Vec<u64>,
    },
    Delete {
        key: u64,
    },
    Expect {
        key: u64,
        /// Required row values; absent = the row must merely exist.
        #[serde(default)]
        values: Option<Vec<u64>>,
        /// Require the row to be absent instead.
        #[serde(default)]
        absent: bool,
    },
}

impl SagaOp {
    fn key(&self) -> u64 {
        match self {
            SagaOp::Put { key, .. } | SagaOp::Delete { key } | SagaOp::Expect { key, .. } => *key,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StepPayload {
    #[serde(default)]
    ops: Vec<SagaOp>,
}

#[derive(Serialize, Deserialize)]
struct ExecReq {
    shard_id: u64,
    ops: Vec<SagaOp>,
    /// Hex versionstamp of the coordinator's step stamp.
    stamp: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ExecErr {
    transient: bool,
    msg: String,
}

/// [`SagaStepExecutor`]: routes a step to the shard owner through the shard
/// map and runs it as a shard-local transaction there.
pub struct EngineStepExecutor {
    core: Arc<Core>,
}

#[async_trait]
impl SagaStepExecutor for EngineStepExecutor {
    async fn execute(&self, req: &StepRequest) -> Result<(), StepError> {
        let payload: StepPayload = serde_json::from_slice(&req.payload)
            .map_err(|e| StepError::Permanent(format!("bad step payload: {e}")))?;
        let table = self
            .core
            .control()
            .ok_or_else(|| StepError::Transient("shard map not ready".into()))?
            .table();
        if table.shards.is_empty() {
            return Err(StepError::Transient("shard map not initialised yet".into()));
        }
        let owner = table
            .shard(req.shard as u64)
            .map(|s| s.owner.clone())
            .ok_or_else(|| StepError::Permanent(format!("no such shard {}", req.shard)))?;
        if owner == self.core.me {
            return self.core.node.exec_local(req.shard as u64, &payload.ops);
        }
        let peer = self.core.peers.get(&owner).ok_or_else(|| {
            StepError::Transient(format!("owner {} unreachable (not a known peer)", owner.0))
        })?;
        let body = serde_json::to_string(&ExecReq {
            shard_id: req.shard as u64,
            ops: payload.ops,
            stamp: Some(hex::encode(req.stamp.to_bytes())),
        })
        .map_err(|e| StepError::Permanent(e.to_string()))?;
        let (code, resp) = http_call(
            &peer.http,
            "POST",
            "/shardnode/exec",
            &body,
            self.core.secret.as_deref(),
            Duration::from_secs(30),
        )
        .await
        .map_err(StepError::Transient)?;
        match code {
            200 => Ok(()),
            409 => {
                let e: ExecErr = serde_json::from_str(&resp).unwrap_or(ExecErr {
                    transient: true,
                    msg: resp,
                });
                if e.transient {
                    Err(StepError::Transient(e.msg))
                } else {
                    Err(StepError::Permanent(e.msg))
                }
            }
            c => Err(StepError::Transient(format!("owner returned {c}: {resp}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SagaStatus {
    pub saga_id: String,
    /// `running`, `committed` or `aborted`.
    pub state: String,
    pub steps: usize,
    pub error: Option<String>,
}

pub struct SagaManager {
    dir: PathBuf,
    coord: Arc<SagaCoordinator<EngineStepExecutor>>,
    status: Mutex<HashMap<String, SagaStatus>>,
    core: Arc<Core>,
}

#[derive(Deserialize)]
struct StepIn {
    id: String,
    shard: u32,
    #[serde(default)]
    forward: serde_json::Value,
    #[serde(default)]
    compensate: serde_json::Value,
    timeout_ms: Option<u64>,
    max_attempts: Option<u32>,
    backoff_ms: Option<u64>,
}

#[derive(Deserialize)]
struct SagaIn {
    saga_id: String,
    steps: Vec<StepIn>,
}

impl SagaManager {
    fn new(core: Arc<Core>, dir: PathBuf, clock: Arc<VersionstampClock>) -> Result<Arc<Self>> {
        std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let exec = Arc::new(EngineStepExecutor { core: core.clone() });
        Ok(Arc::new(SagaManager {
            dir,
            coord: Arc::new(SagaCoordinator::new(exec, clock)),
            status: Mutex::new(HashMap::new()),
            core,
        }))
    }

    fn parse(body: &str) -> Result<SagaDef, String> {
        let req: SagaIn = serde_json::from_str(body).map_err(|e| format!("bad request: {e}"))?;
        let id_ok = !req.saga_id.is_empty()
            && req.saga_id.len() <= 64
            && req
                .saga_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !id_ok {
            return Err("saga_id must be 1-64 chars of [A-Za-z0-9_-]".into());
        }
        if req.steps.is_empty() {
            return Err("a saga needs at least one step".into());
        }
        let mut steps = Vec::new();
        for s in req.steps {
            let mut d = StepDef::new(
                &s.id,
                s.shard,
                &serde_json::to_vec(&s.forward).map_err(|e| e.to_string())?,
                &serde_json::to_vec(&s.compensate).map_err(|e| e.to_string())?,
            );
            if let Some(v) = s.timeout_ms {
                d.timeout_ms = v;
            }
            if let Some(v) = s.max_attempts {
                d.max_attempts = v.max(1);
            }
            if let Some(v) = s.backoff_ms {
                d.backoff_ms = v;
            }
            // Reject malformed op payloads up front, not at execution time.
            for p in [&s.forward, &s.compensate] {
                if !p.is_null() {
                    serde_json::from_value::<StepPayload>(p.clone())
                        .map_err(|e| format!("step {}: bad ops: {e}", s.id))?;
                }
            }
            steps.push(d);
        }
        Ok(SagaDef {
            saga_id: req.saga_id,
            steps,
        })
    }

    fn submit(self: &Arc<Self>, body: &str) -> (u16, String) {
        let def = match Self::parse(body) {
            Ok(d) => d,
            Err(e) => return (400, e),
        };
        let path = self.dir.join(format!("{}.log", def.saga_id));
        {
            let mut st = self.status.lock().unwrap();
            if st.contains_key(&def.saga_id) || path.exists() {
                return (409, format!("saga {} already exists", def.saga_id));
            }
            st.insert(
                def.saga_id.clone(),
                SagaStatus {
                    saga_id: def.saga_id.clone(),
                    state: "running".into(),
                    steps: def.steps.len(),
                    error: None,
                },
            );
        }
        let log = match SagaLog::open(&path) {
            Ok(l) => l,
            Err(e) => {
                self.status.lock().unwrap().remove(&def.saga_id);
                return (500, format!("saga log: {e}"));
            }
        };
        let id = def.saga_id.clone();
        self.spawn_run(id.clone(), log, Some(def));
        (202, serde_json::json!({ "saga_id": id }).to_string())
    }

    fn spawn_run(self: &Arc<Self>, id: String, mut log: SagaLog, def: Option<SagaDef>) {
        let me = Arc::clone(self);
        runtime().spawn(async move {
            let res = me.coord.run(&mut log, def).await;
            let reason = log.records.iter().rev().find_map(|r| match r {
                Record::Failed { idx, reason } => Some(format!("step {idx}: {reason}")),
                _ => None,
            });
            let mut st = me.status.lock().unwrap();
            if let Some(s) = st.get_mut(&id) {
                match res {
                    Ok(SagaOutcome::Committed) => s.state = "committed".into(),
                    Ok(SagaOutcome::Aborted) => {
                        s.state = "aborted".into();
                        s.error = reason;
                    }
                    Err(e) => {
                        s.state = "running".into();
                        s.error = Some(e.to_string());
                    }
                }
            }
        });
    }

    fn get(&self, id: &str) -> (u16, String) {
        match self.status.lock().unwrap().get(id) {
            Some(s) => (200, serde_json::to_string(s).unwrap_or_default()),
            None => (404, format!("unknown saga {id}")),
        }
    }

    /// Resume every unfinished saga found in the log directory (waits for the
    /// shard map so steps do not burn their retries before the node is ready).
    async fn resume_all(self: &Arc<Self>) {
        let Ok(rd) = std::fs::read_dir(&self.dir) else {
            return;
        };
        let mut todo = Vec::new();
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("log") {
                continue;
            }
            let Some(id) = p.file_stem().and_then(|s| s.to_str()).map(String::from) else {
                continue;
            };
            let Ok(log) = SagaLog::open(&p) else { continue };
            let steps = log
                .records
                .iter()
                .find_map(|r| match r {
                    Record::Begin(d) => Some(d.steps.len()),
                    _ => None,
                })
                .unwrap_or(0);
            let fin = log.records.iter().find_map(|r| match r {
                Record::Finished(o) => Some(*o),
                _ => None,
            });
            let state = match fin {
                Some(SagaOutcome::Committed) => "committed",
                Some(SagaOutcome::Aborted) => "aborted",
                None => "running",
            };
            self.status.lock().unwrap().insert(
                id.clone(),
                SagaStatus {
                    saga_id: id.clone(),
                    state: state.into(),
                    steps,
                    error: None,
                },
            );
            if fin.is_none() && steps > 0 {
                todo.push((id, log));
            }
        }
        if todo.is_empty() {
            return;
        }
        for _ in 0..300 {
            if self
                .core
                .control()
                .is_some_and(|c| !c.table().shards.is_empty())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        for (id, log) in todo {
            eprintln!("  sagas     : resuming unfinished saga {id}");
            self.spawn_run(id, log, None);
        }
    }
}

// ---------------------------------------------------------------------------
// the node's extension handler
// ---------------------------------------------------------------------------

struct PeerCfg {
    http: String,
}

struct Plane {
    handle: Arc<RaftHandle>,
    control: Arc<RaftMapControl>,
    table: SharedTable,
    admin: ShardAdmin,
    nodes: Nodes,
}

/// State shared by the HTTP handler, the saga executor and the background loop.
pub struct Core {
    me: NodeId,
    node: Arc<EngineShardNode>,
    peers: HashMap<NodeId, PeerCfg>,
    secret: Option<String>,
    clock: Arc<VersionstampClock>,
    plane: OnceLock<Plane>,
    sagas: OnceLock<Arc<SagaManager>>,
}

impl Core {
    fn control(&self) -> Option<&Arc<RaftMapControl>> {
        self.plane.get().map(|p| &p.control)
    }

    fn authorized(&self, req: &ExtRequest) -> bool {
        match &self.secret {
            Some(s) => req.secret.as_deref() == Some(s.as_str()),
            None => req.loopback,
        }
    }
}

enum Leader {
    Local,
    Remote(String),
    Unknown,
}

impl Core {
    async fn leader(&self) -> Leader {
        let Some(p) = self.plane.get() else {
            return Leader::Unknown;
        };
        let Some(st) = p.handle.status().await else {
            return Leader::Unknown;
        };
        match st.leader_id {
            Some(l) if l == self.me => Leader::Local,
            Some(l) => match self.peers.get(&l) {
                Some(peer) => Leader::Remote(peer.http.clone()),
                None => Leader::Unknown,
            },
            None => Leader::Unknown,
        }
    }
}

#[async_trait]
impl ExtHandler for Core {
    async fn handle(&self, req: ExtRequest) -> Option<(u16, String)> {
        let (m, p) = (req.method.as_str(), req.path.as_str());
        match (m, p) {
            ("GET", "/versionstamp") => {
                return Some(match self.clock.next() {
                    Ok(s) => (
                        200,
                        serde_json::json!({
                            "versionstamp": hex::encode(s.to_bytes()),
                            "epoch": s.epoch, "physical_ms": s.physical_ms,
                            "logical": s.logical, "node": s.node,
                        })
                        .to_string(),
                    ),
                    Err(e) => (500, e.to_string()),
                });
            }
            ("GET", "/shards") => {
                let Some(pl) = self.plane.get() else {
                    return Some((503, "shard plane not running".into()));
                };
                return Some(pl.admin.handle(m, p, &req.body).await);
            }
            _ => {}
        }
        if p.starts_with("/shards/") && m == "POST" {
            if !self.authorized(&req) {
                return Some((
                    403,
                    "shard administration requires loopback or --secret".into(),
                ));
            }
            return Some(match self.leader().await {
                Leader::Local => match self.plane.get() {
                    Some(pl) => pl.admin.handle(m, p, &req.body).await,
                    None => (503, "shard plane not running".into()),
                },
                Leader::Remote(addr) => {
                    match http_call(
                        &addr,
                        m,
                        p,
                        &req.body,
                        self.secret.as_deref(),
                        Duration::from_secs(300),
                    )
                    .await
                    {
                        Ok(r) => r,
                        Err(e) => (502, format!("forwarding to shard-map leader failed: {e}")),
                    }
                }
                Leader::Unknown => (503, "no shard-map leader elected yet".into()),
            });
        }
        if let Some(op) = p.strip_prefix("/shardnode/") {
            if m != "POST" {
                return None;
            }
            if !self.authorized(&req) {
                return Some((403, "shard data plane requires loopback or --secret".into()));
            }
            if op == "exec" {
                return Some(self.exec_remote(&req.body));
            }
            return Some(serve_shard_rpc(&*self.node, op, &req.body).await);
        }
        if p == "/sagas" || p.starts_with("/sagas/") {
            let Some(sagas) = self.sagas.get() else {
                return Some((503, "sagas not running".into()));
            };
            return Some(match (m, p) {
                ("POST", "/sagas") => {
                    if !self.authorized(&req) {
                        (403, "saga submission requires loopback or --secret".into())
                    } else {
                        sagas.submit(&req.body)
                    }
                }
                ("GET", _) => sagas.get(&p["/sagas/".len().min(p.len())..]),
                _ => (404, "unknown saga endpoint".into()),
            });
        }
        None
    }
}

impl Core {
    /// `/shardnode/exec`: a peer's coordinator runs one saga step here.
    fn exec_remote(&self, body: &str) -> (u16, String) {
        let req: ExecReq = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => return (400, format!("bad request: {e}")),
        };
        if let Some(h) = &req.stamp
            && let Ok(b) = hex::decode(h)
            && let Some(s) = Versionstamp::from_bytes(&b)
        {
            // HLC receive rule: our next commit stamp is above the coordinator's.
            let _ = self.clock.observe(s);
        }
        match self.node.exec_local(req.shard_id, &req.ops) {
            Ok(()) => (200, "{}".into()),
            Err(e) => {
                let (transient, msg) = match e {
                    StepError::Transient(m) => (true, m),
                    StepError::Permanent(m) => (false, m),
                };
                (
                    409,
                    serde_json::to_string(&ExecErr { transient, msg }).unwrap_or_default(),
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// startup
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct PeerFile {
    raft_listen: Option<SocketAddr>,
    #[serde(default)]
    peers: Vec<PeerEntry>,
}

#[derive(Deserialize)]
struct PeerEntry {
    id: String,
    raft: SocketAddr,
    http: String,
}

fn resolve_node_id(args: &ServeArgs) -> Result<NodeId> {
    if let Some(s) = args.cluster_node_id.as_deref() {
        return Ok(NodeId(
            uuid::Uuid::parse_str(s).with_context(|| "invalid --cluster-node-id")?,
        ));
    }
    if let Some(i) = super::cluster_cmd::load_identity(&args.dir) {
        return Ok(NodeId(i.node_id));
    }
    let p = args.dir.join("shard_node_id");
    if let Ok(s) = std::fs::read_to_string(&p)
        && let Ok(u) = uuid::Uuid::parse_str(s.trim())
    {
        return Ok(NodeId(u));
    }
    let id = NodeId::new();
    let _ = std::fs::write(&p, id.0.to_string());
    Ok(id)
}

/// Install versionstamps and (with `--cluster-bind`) the shard plane, sagas and
/// their HTTP routes. Single-table mode only; a no-op under `--multi`.
pub fn start(args: &ServeArgs, engine: Option<&Arc<TableEngine>>) -> Result<()> {
    let Some(engine) = engine else {
        if args.cluster_bind.is_some() {
            eprintln!("  shards    : disabled under --multi (single-table mode only)");
        }
        return Ok(());
    };
    let me = resolve_node_id(args)?;

    // ── F8: versionstamp clock + commit stamper ──────────────────────────────
    let epoch0 = EPOCH_SRC
        .get()
        .map(|f| f().min(u16::MAX as u64) as u16)
        .unwrap_or(0);
    let clock = Arc::new(
        VersionstampClock::open(
            hash16(&me),
            epoch0,
            Some(args.dir.join("versionstamp.clock")),
            system_wall_clock(),
        )
        .with_context(|| "open versionstamp clock")?,
    );
    let _ = CLOCK.set(Arc::clone(&clock));
    // Replication followers merge the primary's commit stamps into this clock
    // and store the same stamp on their own CDC events.
    vectorvolt_replication::set_stamp_observer(Arc::new(|b: &[u8]| {
        observe_remote_stamp(b);
    }));
    {
        let c = Arc::clone(&clock);
        let last = AtomicU64::new(epoch0 as u64);
        engine.set_commit_stamper(Some(Arc::new(move |_lsn| {
            if let Some(src) = EPOCH_SRC.get() {
                let e = src().min(u16::MAX as u64);
                if e > last.load(Ordering::Relaxed) {
                    last.store(e, Ordering::Relaxed);
                    let _ = c.set_epoch(e as u16);
                }
            }
            c.next().ok().map(|s| s.to_bytes())
        })));
    }

    let Some(bind) = args.cluster_bind else {
        return Ok(());
    };

    // ── F7: shard plane ──────────────────────────────────────────────────────
    let pf: PeerFile = match std::fs::read(args.dir.join("shard_cluster.json")) {
        Ok(b) => serde_json::from_slice(&b).with_context(|| "parse shard_cluster.json")?,
        Err(_) => PeerFile::default(),
    };
    let mut peers = HashMap::new();
    let mut raft_peers = Vec::new();
    for p in &pf.peers {
        let id = NodeId(uuid::Uuid::parse_str(&p.id).with_context(|| "peer id")?);
        if id == me {
            continue;
        }
        raft_peers.push((id.clone(), p.raft));
        peers.insert(
            id,
            PeerCfg {
                http: p.http.clone(),
            },
        );
    }
    let node = EngineShardNode::new(me.clone(), Arc::clone(engine));
    if !raft_peers.is_empty() {
        node.enforce_ownership();
    }
    let core = Arc::new(Core {
        me: me.clone(),
        node: Arc::clone(&node),
        peers,
        secret: args.secret.clone(),
        clock: Arc::clone(&clock),
        plane: OnceLock::new(),
        sagas: OnceLock::new(),
    });
    let raft_listen = pf.raft_listen.unwrap_or(bind);
    let dir = args.dir.clone();
    let c2 = Arc::clone(&core);
    let rt = runtime();
    rt.block_on(start_plane(c2, dir.clone(), raft_listen, raft_peers))
        .with_context(|| "start shard plane")?;
    let sagas = SagaManager::new(Arc::clone(&core), dir.join("sagas"), clock)?;
    let _ = core.sagas.set(Arc::clone(&sagas));
    rt.spawn({
        let sagas = Arc::clone(&sagas);
        async move { sagas.resume_all().await }
    });
    install_ext(core);
    eprintln!(
        "  shards    : raft {} ({} peer(s)); /shards*, /sagas*, /versionstamp on the control port",
        raft_listen,
        pf.peers.len()
    );
    Ok(())
}

async fn start_plane(
    core: Arc<Core>,
    dir: PathBuf,
    raft_listen: SocketAddr,
    raft_peers: Vec<(NodeId, SocketAddr)>,
) -> Result<()> {
    let me = core.me.clone();
    let transport = Arc::new(
        TcpTransport::bind(
            me.clone(),
            raft_listen,
            raft_peers.iter().cloned().collect(),
            TcpConfig::default(),
        )
        .await
        .with_context(|| format!("bind shard-map raft on {raft_listen}"))?,
    );
    let sm = ShardMapSm::new();
    let (table, results) = (Arc::clone(&sm.table), Arc::clone(&sm.results));
    let rdir = dir.join("shardmap").join("raft");
    std::fs::create_dir_all(&rdir)?;
    let node = RaftNode::open(
        me.clone(),
        raft_peers.iter().map(|p| p.0.clone()).collect(),
        RaftConfig {
            election_ticks: 12,
            heartbeat_ticks: 2,
            snapshot_threshold: 512,
            seed: me.0.as_u128() as u64,
            ..RaftConfig::default()
        },
        rdir,
        sm,
    )
    .map_err(|e| anyhow::anyhow!("open shard-map raft: {e}"))?;
    let handle = Arc::new(server::spawn(node, transport, Duration::from_millis(25)));
    let control = Arc::new(RaftMapControl::new(vec![RaftMember {
        handle: Arc::clone(&handle),
        table: Arc::clone(&table),
        results,
    }]));
    let mut all: Vec<Arc<dyn ShardNode>> = vec![core.node.clone()];
    for (id, p) in &core.peers {
        all.push(Arc::new(RemoteShardNode::new(
            id.clone(),
            p.http.clone(),
            core.secret.clone(),
        )));
    }
    let nodes = Nodes::new(all);
    // Every migration phase is journaled under `<dir>/shard_migrations`; a
    // coordinator restart repairs whatever the crash left behind (below).
    let journal = Arc::new(
        MigrationJournal::open(dir.join("shard_migrations"))
            .with_context(|| "open shard migration journal")?,
    );
    let admin = ShardAdmin::new(
        control.clone() as Arc<dyn MapControl>,
        nodes.clone(),
        MigrationConfig {
            journal: Some(Arc::clone(&journal)),
            ..MigrationConfig::default()
        },
        Rebalancer::new(RebalanceConfig::default()),
    );
    let _ = core.plane.set(Plane {
        handle: Arc::clone(&handle),
        control: Arc::clone(&control),
        table,
        admin,
        nodes: nodes.clone(),
    });

    // Initialise the map once a leader exists; host the shards this node owns
    // (also after a restart, when the map is rebuilt from the Raft log).
    let c = core.clone();
    tokio::spawn(async move {
        let mut tries = 0u32;
        loop {
            tokio::time::sleep(Duration::from_millis(150)).await;
            let Some(pl) = c.plane.get() else { continue };
            let empty = pl.table.lock().unwrap().shards.is_empty();
            if empty {
                let leader = pl
                    .handle
                    .status()
                    .await
                    .is_some_and(|s| s.role == RaftRole::Leader);
                if leader {
                    // A concurrent Init loses with "already initialised".
                    let r = bootstrap(pl.control.as_ref(), &pl.nodes, &c.me).await;
                    tries += 1;
                    if r.is_err() && tries > 20 {
                        eprintln!("  shards    : initial shard map not created: {r:?}");
                        return;
                    }
                }
                continue;
            }
            let mine = pl.table.lock().unwrap().shards_of(&c.me);
            for s in mine {
                if !c.node.is_hosted(s.shard_id) {
                    let _ = c.node.host_shard(s.shard_id, s.lo, s.hi, true).await;
                }
            }
            // Resume or roll back migrations this coordinator left in flight
            // (retry while a peer needed for the repair is still down).
            for _ in 0..60 {
                if journal.load_all().is_empty() {
                    break;
                }
                let done =
                    recover_migrations(pl.control.as_ref(), &pl.nodes, journal.as_ref()).await;
                for r in &done {
                    eprintln!("  shards    : migration recovery: {r:?}");
                }
                if !done.iter().any(|r| matches!(r, Recovery::Pending(_))) {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            return;
        }
    });
    Ok(())
}
