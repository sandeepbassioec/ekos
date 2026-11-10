//! Shard split, live migration and node drain (gap F7).
//!
//! Control plane: every shard-map change is a [`ShardCmd`] proposed through
//! Raft ([`MapControl`]); all nodes apply it to their [`ShardTable`] at one
//! `version`.
//!
//! Data plane: a minimal shard-scoped node API ([`ShardNode`]); production uses
//! the engine-backed implementation in `vvctl`, tests use the
//! in-memory test double ([`MemShardNode`]). Live migration is:
//! begin (Raft) -> snapshot copy -> change streaming -> short write freeze on
//! the shard -> final catch-up -> map switch (Raft) -> release source.
//! Reads and writes are served by the source throughout; clients that hit the
//! frozen/released source retry against the refreshed map.

use crate::node::NodeId;
use crate::raft::node::RaftError;
use crate::raft::server::RaftHandle;
use crate::shard::{
    ShardCmd, ShardRange, ShardTable, SharedResults, SharedTable, median_split_key,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Error, PartialEq)]
pub enum DataError {
    #[error("node does not own this key")]
    NotOwner,
    #[error("shard is frozen for cutover")]
    Frozen,
    #[error("node is down")]
    Down,
    #[error("shard {0} not hosted here")]
    NotHosted(u64),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("shard map: {0}")]
    Control(String),
    #[error("map change rejected: {0}")]
    Rejected(String),
    #[error("data plane: {0}")]
    Data(#[from] DataError),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("timed out: {0}")]
    Timeout(String),
    #[error("migration aborted and rolled back: {0}")]
    Aborted(String),
    /// The coordinator died (fault injection via [`MigrationConfig::crash_after`]):
    /// nothing was rolled back; [`recover_migrations`] must repair the state.
    #[error("coordinator crashed after phase {0:?}")]
    Crashed(MigrationPhase),
    #[error("migration journal: {0}")]
    Journal(String),
}

// ---------------------------------------------------------------------------
// Data plane
// ---------------------------------------------------------------------------

/// One change in a shard's capture log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub seq: u64,
    pub key: u64,
    /// `None` = delete.
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardLoad {
    pub shard_id: u64,
    pub keys: u64,
    /// Total operations served since start (monotonic).
    pub ops_total: u64,
}

/// Shard-scoped data API of one node.
#[async_trait]
pub trait ShardNode: Send + Sync {
    fn id(&self) -> NodeId;
    /// Host an (empty) shard; `serving=false` hosts it as an import target.
    async fn host_shard(
        &self,
        shard_id: u64,
        lo: u64,
        hi: u64,
        serving: bool,
    ) -> Result<(), DataError>;
    async fn put(&self, key: u64, value: Vec<u8>) -> Result<(), DataError>;
    async fn delete(&self, key: u64) -> Result<(), DataError>;
    async fn get(&self, key: u64) -> Result<Option<Vec<u8>>, DataError>;
    /// Start recording changes; returns the sequence number to stream from.
    async fn begin_capture(&self, shard_id: u64) -> Result<u64, DataError>;
    async fn scan(
        &self,
        shard_id: u64,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<(u64, Vec<u8>)>, DataError>;
    async fn changes_since(
        &self,
        shard_id: u64,
        seq: u64,
        limit: usize,
    ) -> Result<Vec<Change>, DataError>;
    async fn apply_import(
        &self,
        shard_id: u64,
        entries: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), DataError>;
    async fn apply_changes(&self, shard_id: u64, changes: Vec<Change>) -> Result<(), DataError>;
    /// Reject writes to the shard (reads continue).
    async fn freeze(&self, shard_id: u64) -> Result<(), DataError>;
    /// Source-side rollback: stop capture and resume writes.
    async fn rollback_source(&self, shard_id: u64) -> Result<(), DataError>;
    /// Target-side: start serving an imported shard.
    async fn activate(&self, shard_id: u64) -> Result<(), DataError>;
    /// Source-side: drop the shard after cutover (writes now get `NotOwner`).
    async fn release(&self, shard_id: u64) -> Result<(), DataError>;
    /// Target-side rollback: drop an imported shard.
    async fn discard(&self, shard_id: u64) -> Result<(), DataError>;
    /// Split a locally hosted shard: keys `>= at` move to `new_id`.
    async fn split_local(&self, shard_id: u64, at: u64, new_id: u64) -> Result<(), DataError>;
    async fn median_key(&self, shard_id: u64) -> Result<Option<u64>, DataError>;
    async fn loads(&self) -> Result<Vec<ShardLoad>, DataError>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Serving,
    Frozen,
    Importing,
}

struct Hosted {
    lo: u64,
    hi: u64,
    st: St,
    data: BTreeMap<u64, Vec<u8>>,
    log: Vec<Change>,
    seq: u64,
    capturing: bool,
    ops: u64,
}

impl Hosted {
    fn new(lo: u64, hi: u64, st: St) -> Self {
        Hosted {
            lo,
            hi,
            st,
            data: BTreeMap::new(),
            log: Vec::new(),
            seq: 0,
            capturing: false,
            ops: 0,
        }
    }
    fn contains(&self, k: u64) -> bool {
        k >= self.lo && (k < self.hi || self.hi == u64::MAX)
    }
    fn record(&mut self, key: u64, value: Option<Vec<u8>>) {
        if self.capturing {
            self.seq += 1;
            self.log.push(Change {
                seq: self.seq,
                key,
                value,
            });
        }
    }
}

/// **TEST DOUBLE ONLY — never used by `vvctl serve`.** In-memory [`ShardNode`]
/// with failure injection. The production data plane is the engine-backed
/// shard node in `vvctl` (`serve_cluster_ext`), reached across nodes through
/// [`crate::shard_rpc::RemoteShardNode`].
pub struct MemShardNode {
    id: NodeId,
    shards: Mutex<HashMap<u64, Hosted>>,
    down: AtomicBool,
    /// When >= 0: remaining successful import/stream applies before the node
    /// "dies". -1 disables.
    fail_after: AtomicI64,
}

impl MemShardNode {
    pub fn new(id: NodeId) -> Arc<Self> {
        Arc::new(MemShardNode {
            id,
            shards: Mutex::new(HashMap::new()),
            down: AtomicBool::new(false),
            fail_after: AtomicI64::new(-1),
        })
    }

    pub fn set_down(&self, down: bool) {
        self.down.store(down, Ordering::SeqCst);
    }

    /// Die after `n` more import/stream applies.
    pub fn fail_after_applies(&self, n: i64) {
        self.fail_after.store(n, Ordering::SeqCst);
    }

    fn alive(&self) -> Result<(), DataError> {
        if self.down.load(Ordering::SeqCst) {
            Err(DataError::Down)
        } else {
            Ok(())
        }
    }

    fn apply_gate(&self) -> Result<(), DataError> {
        self.alive()?;
        let n = self.fail_after.load(Ordering::SeqCst);
        if n == 0 {
            self.down.store(true, Ordering::SeqCst);
            return Err(DataError::Down);
        }
        if n > 0 {
            self.fail_after.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Test helper: ids of hosted shards.
    pub fn hosted_ids(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.shards.lock().unwrap().keys().copied().collect();
        v.sort_unstable();
        v
    }

    /// Test helper: all `(key, value)` pairs hosted (any state).
    pub fn dump(&self) -> Vec<(u64, Vec<u8>)> {
        let g = self.shards.lock().unwrap();
        let mut v: Vec<_> = g
            .values()
            .flat_map(|s| s.data.iter().map(|(k, v)| (*k, v.clone())))
            .collect();
        v.sort();
        v
    }

    pub fn total_keys(&self) -> usize {
        self.shards
            .lock()
            .unwrap()
            .values()
            .map(|s| s.data.len())
            .sum()
    }

    pub fn is_frozen(&self, shard_id: u64) -> bool {
        self.shards
            .lock()
            .unwrap()
            .get(&shard_id)
            .is_some_and(|s| s.st == St::Frozen)
    }

    pub fn is_capturing(&self, shard_id: u64) -> bool {
        self.shards
            .lock()
            .unwrap()
            .get(&shard_id)
            .is_some_and(|s| s.capturing)
    }
}

fn find_mut(g: &mut HashMap<u64, Hosted>, key: u64) -> Option<&mut Hosted> {
    g.values_mut().find(|s| s.contains(key))
}

#[async_trait]
impl ShardNode for MemShardNode {
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
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        if g.get(&shard_id).is_some_and(|e| e.st != St::Importing) {
            return Err(DataError::Other(format!("shard {shard_id} already hosted")));
        }
        let st = if serving { St::Serving } else { St::Importing };
        g.insert(shard_id, Hosted::new(lo, hi, st));
        Ok(())
    }

    async fn put(&self, key: u64, value: Vec<u8>) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = find_mut(&mut g, key).ok_or(DataError::NotOwner)?;
        match s.st {
            St::Importing => return Err(DataError::NotOwner),
            St::Frozen => return Err(DataError::Frozen),
            St::Serving => {}
        }
        s.ops += 1;
        s.data.insert(key, value.clone());
        s.record(key, Some(value));
        Ok(())
    }

    async fn delete(&self, key: u64) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = find_mut(&mut g, key).ok_or(DataError::NotOwner)?;
        match s.st {
            St::Importing => return Err(DataError::NotOwner),
            St::Frozen => return Err(DataError::Frozen),
            St::Serving => {}
        }
        s.ops += 1;
        s.data.remove(&key);
        s.record(key, None);
        Ok(())
    }

    async fn get(&self, key: u64) -> Result<Option<Vec<u8>>, DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = find_mut(&mut g, key).ok_or(DataError::NotOwner)?;
        if s.st == St::Importing {
            return Err(DataError::NotOwner);
        }
        s.ops += 1;
        Ok(s.data.get(&key).cloned())
    }

    async fn begin_capture(&self, shard_id: u64) -> Result<u64, DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        s.capturing = true;
        s.log.clear();
        Ok(s.seq)
    }

    async fn scan(
        &self,
        shard_id: u64,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<(u64, Vec<u8>)>, DataError> {
        self.alive()?;
        let g = self.shards.lock().unwrap();
        let s = g.get(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        let it: Box<dyn Iterator<Item = (&u64, &Vec<u8>)>> = match after {
            Some(a) => Box::new(
                s.data
                    .range((std::ops::Bound::Excluded(a), std::ops::Bound::Unbounded)),
            ),
            None => Box::new(s.data.iter()),
        };
        Ok(it.take(limit).map(|(k, v)| (*k, v.clone())).collect())
    }

    async fn changes_since(
        &self,
        shard_id: u64,
        seq: u64,
        limit: usize,
    ) -> Result<Vec<Change>, DataError> {
        self.alive()?;
        let g = self.shards.lock().unwrap();
        let s = g.get(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        Ok(s.log
            .iter()
            .filter(|c| c.seq > seq)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn apply_import(
        &self,
        shard_id: u64,
        entries: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), DataError> {
        self.apply_gate()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        for (k, v) in entries {
            s.data.insert(k, v);
        }
        Ok(())
    }

    async fn apply_changes(&self, shard_id: u64, changes: Vec<Change>) -> Result<(), DataError> {
        self.apply_gate()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        for c in changes {
            match c.value {
                Some(v) => {
                    s.data.insert(c.key, v);
                }
                None => {
                    s.data.remove(&c.key);
                }
            }
        }
        Ok(())
    }

    async fn freeze(&self, shard_id: u64) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        if s.st == St::Serving {
            s.st = St::Frozen;
        }
        Ok(())
    }

    async fn rollback_source(&self, shard_id: u64) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        s.capturing = false;
        s.log.clear();
        if s.st == St::Frozen {
            s.st = St::Serving;
        }
        Ok(())
    }

    async fn activate(&self, shard_id: u64) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        s.st = St::Serving;
        Ok(())
    }

    async fn release(&self, shard_id: u64) -> Result<(), DataError> {
        self.alive()?;
        self.shards.lock().unwrap().remove(&shard_id);
        Ok(())
    }

    async fn discard(&self, shard_id: u64) -> Result<(), DataError> {
        // Allowed even when "down" would be wrong for a dead node; a dead
        // node simply keeps its garbage until it restarts empty.
        self.alive()?;
        self.shards.lock().unwrap().remove(&shard_id);
        Ok(())
    }

    async fn split_local(&self, shard_id: u64, at: u64, new_id: u64) -> Result<(), DataError> {
        self.alive()?;
        let mut g = self.shards.lock().unwrap();
        let s = g.get_mut(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        if s.capturing {
            return Err(DataError::Other("shard is being migrated".into()));
        }
        if at <= s.lo || (at >= s.hi && s.hi != u64::MAX) {
            return Err(DataError::Other("split point out of range".into()));
        }
        let hi_data = s.data.split_off(&at);
        let mut hi = Hosted::new(at, s.hi, s.st);
        hi.data = hi_data;
        s.hi = at;
        g.insert(new_id, hi);
        Ok(())
    }

    async fn median_key(&self, shard_id: u64) -> Result<Option<u64>, DataError> {
        self.alive()?;
        let g = self.shards.lock().unwrap();
        let s = g.get(&shard_id).ok_or(DataError::NotHosted(shard_id))?;
        let keys: Vec<u64> = s.data.keys().copied().collect();
        Ok(median_split_key(&keys, s.lo))
    }

    async fn loads(&self) -> Result<Vec<ShardLoad>, DataError> {
        self.alive()?;
        let g = self.shards.lock().unwrap();
        let mut v: Vec<_> = g
            .iter()
            .filter(|(_, s)| s.st != St::Importing)
            .map(|(id, s)| ShardLoad {
                shard_id: *id,
                keys: s.data.len() as u64,
                ops_total: s.ops,
            })
            .collect();
        v.sort_by_key(|l| l.shard_id);
        Ok(v)
    }
}

/// Registry of the data-plane nodes reachable from the coordinator.
#[derive(Clone, Default)]
pub struct Nodes(Arc<HashMap<NodeId, Arc<dyn ShardNode>>>);

impl Nodes {
    pub fn new(nodes: Vec<Arc<dyn ShardNode>>) -> Self {
        Nodes(Arc::new(nodes.into_iter().map(|n| (n.id(), n)).collect()))
    }
    pub fn get(&self, id: &NodeId) -> Result<Arc<dyn ShardNode>, MigrationError> {
        self.0
            .get(id)
            .cloned()
            .ok_or_else(|| MigrationError::NotFound(format!("node {}", id.0)))
    }
    pub fn ids(&self) -> Vec<NodeId> {
        let mut v: Vec<NodeId> = self.0.keys().cloned().collect();
        v.sort_by_key(|n| n.0);
        v
    }
}

// ---------------------------------------------------------------------------
// Control plane (Raft)
// ---------------------------------------------------------------------------

/// Commits shard-map changes and exposes the agreed table.
#[async_trait]
pub trait MapControl: Send + Sync {
    /// Propose `cmd`; returns once it is committed and applied, or the reason
    /// the state machine rejected it.
    async fn commit(&self, cmd: ShardCmd) -> Result<(), MigrationError>;
    /// Latest applied table.
    fn table(&self) -> ShardTable;
}

/// One Raft member as seen by [`RaftMapControl`] (in-process deployments).
pub struct RaftMember {
    pub handle: Arc<RaftHandle>,
    pub table: SharedTable,
    pub results: SharedResults,
}

/// [`MapControl`] over the public Raft handle API: proposes on whichever
/// member is leader.
pub struct RaftMapControl {
    members: Vec<RaftMember>,
    timeout: Duration,
}

impl RaftMapControl {
    pub fn new(members: Vec<RaftMember>) -> Self {
        RaftMapControl {
            members,
            timeout: Duration::from_secs(5),
        }
    }
}

#[async_trait]
impl MapControl for RaftMapControl {
    async fn commit(&self, cmd: ShardCmd) -> Result<(), MigrationError> {
        let bytes = serde_json::to_vec(&cmd).map_err(|e| MigrationError::Control(e.to_string()))?;
        let deadline = Instant::now() + self.timeout;
        loop {
            for m in &self.members {
                let idx = match m.handle.propose(bytes.clone()).await {
                    Ok(i) => i,
                    Err(RaftError::NotLeader(_)) | Err(RaftError::Io(_)) => continue,
                    Err(e) => return Err(MigrationError::Control(e.to_string())),
                };
                loop {
                    if let Some(r) = m.results.lock().unwrap().get(&idx).cloned() {
                        return r.map_err(MigrationError::Rejected);
                    }
                    let applied = m.handle.status().await.map(|s| s.last_applied).unwrap_or(0);
                    if applied >= idx && !m.results.lock().unwrap().contains_key(&idx) {
                        return Err(MigrationError::Control("proposal superseded".into()));
                    }
                    if Instant::now() > deadline {
                        return Err(MigrationError::Timeout("raft commit".into()));
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            if Instant::now() > deadline {
                return Err(MigrationError::Timeout("no raft leader".into()));
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    fn table(&self) -> ShardTable {
        self.members
            .iter()
            .map(|m| m.table.lock().unwrap().clone())
            .max_by_key(|t| t.version)
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Client (routing with redirect)
// ---------------------------------------------------------------------------

/// Routes reads/writes through the agreed shard map. Frozen or moved shards
/// are retried against the refreshed map (the redirect).
pub struct ShardClient {
    control: Arc<dyn MapControl>,
    nodes: Nodes,
    timeout: Duration,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("no shard owns key {0}")]
    NoShard(u64),
    #[error("timed out routing key {0}: {1}")]
    Timeout(u64, DataError),
}

impl ShardClient {
    pub fn new(control: Arc<dyn MapControl>, nodes: Nodes) -> Self {
        ShardClient {
            control,
            nodes,
            timeout: Duration::from_secs(5),
        }
    }

    async fn route<T, F, Fut>(&self, key: u64, op: F) -> Result<T, ClientError>
    where
        F: Fn(Arc<dyn ShardNode>) -> Fut,
        Fut: std::future::Future<Output = Result<T, DataError>>,
    {
        let deadline = Instant::now() + self.timeout;
        let mut last = DataError::NotOwner;
        loop {
            let table = self.control.table();
            let owner = table
                .shard_for(key)
                .map(|s| s.owner.clone())
                .ok_or(ClientError::NoShard(key))?;
            if let Ok(node) = self.nodes.get(&owner) {
                match op(node).await {
                    Ok(v) => return Ok(v),
                    Err(e) => last = e,
                }
            }
            if Instant::now() > deadline {
                return Err(ClientError::Timeout(key, last));
            }
            tokio::time::sleep(Duration::from_micros(500)).await;
        }
    }

    pub async fn put(&self, key: u64, value: Vec<u8>) -> Result<(), ClientError> {
        self.route(key, |n| {
            let v = value.clone();
            async move { n.put(key, v).await }
        })
        .await
    }

    pub async fn delete(&self, key: u64) -> Result<(), ClientError> {
        self.route(key, |n| async move { n.delete(key).await })
            .await
    }

    pub async fn get(&self, key: u64) -> Result<Option<Vec<u8>>, ClientError> {
        self.route(key, |n| async move { n.get(key).await }).await
    }
}

// ---------------------------------------------------------------------------
// Operations: bootstrap, split, migrate, drain
// ---------------------------------------------------------------------------

/// How far a live migration got. Persisted in the [`MigrationJournal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// Recorded before `BeginMove` is proposed (the map may or may not have it).
    Intent,
    /// `BeginMove` committed; target not yet populated.
    Begun,
    /// Snapshot copy to the target in progress / done.
    Copying,
    /// Change streaming in progress.
    Streaming,
    /// Source frozen (final catch-up).
    Frozen,
    /// Target activated; map still names the source.
    Activated,
    /// `Move` committed: the target owns the shard; source cleanup pending.
    Switched,
}

/// Durable record of one in-flight migration (one file per shard).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub shard_id: u64,
    pub lo: u64,
    pub hi: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub phase: MigrationPhase,
    pub copied_keys: u64,
    /// Last key copied by the snapshot phase.
    pub last_key: Option<u64>,
    pub streamed_changes: u64,
}

/// Crash-safe store of [`MigrationRecord`]s (`shard-<id>.json`, written via a
/// synced temp file + rename).
#[derive(Debug)]
pub struct MigrationJournal {
    dir: std::path::PathBuf,
}

impl MigrationJournal {
    pub fn open(dir: impl Into<std::path::PathBuf>) -> Result<Self, MigrationError> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir).map_err(|e| MigrationError::Journal(e.to_string()))?;
        Ok(MigrationJournal { dir })
    }

    fn path(&self, shard_id: u64) -> std::path::PathBuf {
        self.dir.join(format!("shard-{shard_id}.json"))
    }

    pub fn save(&self, rec: &MigrationRecord) -> Result<(), MigrationError> {
        use std::io::Write;
        let io = |e: std::io::Error| MigrationError::Journal(e.to_string());
        let tmp = self.dir.join(format!("shard-{}.json.tmp", rec.shard_id));
        let bytes = serde_json::to_vec(rec).map_err(|e| MigrationError::Journal(e.to_string()))?;
        let mut f = std::fs::File::create(&tmp).map_err(io)?;
        f.write_all(&bytes).map_err(io)?;
        f.sync_all().map_err(io)?;
        drop(f);
        std::fs::rename(&tmp, self.path(rec.shard_id)).map_err(io)?;
        // Make the rename durable (best effort: not supported on every platform).
        if let Ok(d) = std::fs::File::open(&self.dir) {
            let _ = d.sync_all();
        }
        Ok(())
    }

    pub fn clear(&self, shard_id: u64) -> Result<(), MigrationError> {
        match std::fs::remove_file(self.path(shard_id)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(MigrationError::Journal(e.to_string())),
        }
    }

    /// Every in-flight record (unparseable files are skipped).
    pub fn load_all(&self) -> Vec<MigrationRecord> {
        let mut v = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "json")
                    && let Ok(b) = std::fs::read(&p)
                    && let Ok(r) = serde_json::from_slice::<MigrationRecord>(&b)
                {
                    v.push(r);
                }
            }
        }
        v.sort_by_key(|r| r.shard_id);
        v
    }
}

#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// When set, every phase is persisted so [`recover_migrations`] can repair
    /// a migration whose coordinator died.
    pub journal: Option<Arc<MigrationJournal>>,
    /// Fault injection: behave as if the coordinator process died right after
    /// this phase was recorded (no rollback happens).
    pub crash_after: Option<MigrationPhase>,
    /// Keys per snapshot chunk.
    pub chunk: usize,
    /// Changes per streaming batch.
    pub batch: usize,
    /// Freeze once at most this many changes remain un-streamed.
    pub freeze_threshold: usize,
    /// Maximum streaming rounds before freezing regardless.
    pub max_rounds: usize,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        MigrationConfig {
            journal: None,
            crash_after: None,
            chunk: 512,
            batch: 512,
            freeze_threshold: 32,
            max_rounds: 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub shard_id: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub copied_keys: u64,
    pub streamed_changes: u64,
    /// Time writes were rejected: freeze until source release.
    pub cutover: Duration,
}

/// Create the initial shard on `owner`.
pub async fn bootstrap(
    control: &dyn MapControl,
    nodes: &Nodes,
    owner: &NodeId,
) -> Result<(), MigrationError> {
    control
        .commit(ShardCmd::Init {
            owner: owner.clone(),
        })
        .await?;
    let t = control.table();
    for s in t.shards_of(owner) {
        nodes
            .get(owner)?
            .host_shard(s.shard_id, s.lo, s.hi, true)
            .await?;
    }
    Ok(())
}

/// Split `shard_id` at `at` (or the owner's median key) through Raft, then
/// partition the owner's local data. Returns `(lo, hi)` shards.
pub async fn split_shard(
    control: &dyn MapControl,
    nodes: &Nodes,
    shard_id: u64,
    at: Option<u64>,
) -> Result<(ShardRange, ShardRange), MigrationError> {
    let t = control.table();
    let r = t
        .shard(shard_id)
        .cloned()
        .ok_or_else(|| MigrationError::NotFound(format!("shard {shard_id}")))?;
    let owner = nodes.get(&r.owner)?;
    let at = match at {
        Some(a) => a,
        None => owner
            .median_key(shard_id)
            .await?
            .ok_or_else(|| MigrationError::Rejected("no median key (shard too small)".into()))?,
    };
    control.commit(ShardCmd::Split { shard_id, at }).await?;
    let t = control.table();
    let new = t
        .shard_for(at)
        .cloned()
        .ok_or_else(|| MigrationError::NotFound("new shard".into()))?;
    owner.split_local(shard_id, at, new.shard_id).await?;
    let lo = t
        .shard(shard_id)
        .cloned()
        .ok_or_else(|| MigrationError::NotFound("lo shard".into()))?;
    Ok((lo, new))
}

/// Live-migrate `shard_id` to node `to`. On any failure before the map
/// switch the migration is rolled back and the source remains the owner.
pub async fn migrate_shard(
    control: &dyn MapControl,
    nodes: &Nodes,
    shard_id: u64,
    to: &NodeId,
    cfg: &MigrationConfig,
) -> Result<MigrationReport, MigrationError> {
    let r = control
        .table()
        .shard(shard_id)
        .cloned()
        .ok_or_else(|| MigrationError::NotFound(format!("shard {shard_id}")))?;
    let from = r.owner.clone();
    if &from == to {
        return Err(MigrationError::Rejected("target already owns shard".into()));
    }
    let src = nodes.get(&from)?;
    let dst = nodes.get(to)?;
    let mut t = Tracker {
        cfg,
        rec: MigrationRecord {
            shard_id,
            lo: r.lo,
            hi: r.hi,
            from: from.clone(),
            to: to.clone(),
            phase: MigrationPhase::Intent,
            copied_keys: 0,
            last_key: None,
            streamed_changes: 0,
        },
    };
    // Write-ahead: the intent is durable before the map learns of the move.
    t.phase(MigrationPhase::Intent)?;
    if let Err(e) = control
        .commit(ShardCmd::BeginMove {
            shard_id,
            to: to.clone(),
        })
        .await
    {
        // A rejected proposal changed nothing; an unknown outcome (timeout)
        // keeps the journal so recovery can undo a BeginMove that did commit.
        if matches!(e, MigrationError::Rejected(_) | MigrationError::NotFound(_)) {
            t.clear();
        }
        return Err(e);
    }
    t.phase(MigrationPhase::Begun)?;
    match run_migration(control, &r, &src, &dst, to, &mut t).await {
        Ok(rep) => Ok(rep),
        // The coordinator "died": leave everything exactly as it is.
        Err(e @ MigrationError::Crashed(_)) => Err(e),
        Err(e) => {
            let _ = src.rollback_source(shard_id).await;
            let _ = dst.discard(shard_id).await;
            let abort = control.commit(ShardCmd::AbortMove { shard_id }).await;
            match abort {
                Ok(()) => {
                    t.clear();
                    Err(MigrationError::Aborted(e.to_string()))
                }
                Err(a) => Err(MigrationError::Aborted(format!(
                    "{e}; additionally AbortMove failed: {a}"
                ))),
            }
        }
    }
}

/// What [`recover_migrations`] did for one journaled migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recovery {
    /// The map switch had not committed: the source stays the owner, the
    /// partial target copy was dropped.
    RolledBack(u64),
    /// The map switch had committed: the target owns the shard, the leftover
    /// source copy was dropped.
    Completed(u64),
    /// A node needed for the repair is unreachable; the journal entry stays and
    /// the repair is retried on the next call.
    Pending(u64),
}

/// Repair every migration left in `journal` by a coordinator that died.
///
/// The shard map is the authority on ownership (the journal phase can lag one
/// step behind it): if it already names the target, the switch committed and
/// the source's copy is cleaned up; otherwise the migration is rolled back to
/// the source (unfreeze, abort the pending move, drop the target's partial
/// data). Both repairs are idempotent. Hosting the shard as an import before
/// the cleanup makes it work on a node that restarted and lost its in-memory
/// shard registry but still holds the rows.
pub async fn recover_migrations(
    control: &dyn MapControl,
    nodes: &Nodes,
    journal: &MigrationJournal,
) -> Vec<Recovery> {
    let mut out = Vec::new();
    for rec in journal.load_all() {
        let id = rec.shard_id;
        let table = control.table();
        let owner = table.shard(id).map(|s| s.owner.clone());
        let (Ok(src), Ok(dst)) = (nodes.get(&rec.from), nodes.get(&rec.to)) else {
            out.push(Recovery::Pending(id));
            continue;
        };
        if owner.as_ref() == Some(&rec.to) {
            // Switched: finish the cleanup on the source.
            let _ = dst.activate(id).await;
            let _ = src.host_shard(id, rec.lo, rec.hi, false).await;
            if src.release(id).await.is_ok() {
                let _ = journal.clear(id);
                out.push(Recovery::Completed(id));
            } else {
                out.push(Recovery::Pending(id));
            }
            continue;
        }
        // Not switched: roll back to the source.
        if table.shard(id).is_none() {
            let _ = journal.clear(id);
            continue;
        }
        let unfroze = src.rollback_source(id).await;
        let _ = dst.host_shard(id, rec.lo, rec.hi, false).await;
        let dropped = dst.discard(id).await;
        if unfroze.is_err() || dropped.is_err() {
            out.push(Recovery::Pending(id));
            continue;
        }
        match control.commit(ShardCmd::AbortMove { shard_id: id }).await {
            // "Rejected": no move is pending (BeginMove never committed).
            Ok(()) | Err(MigrationError::Rejected(_)) => {
                let _ = journal.clear(id);
                out.push(Recovery::RolledBack(id));
            }
            Err(_) => out.push(Recovery::Pending(id)),
        }
    }
    out
}

/// Persists a migration's phases and injects coordinator crashes.
struct Tracker<'a> {
    cfg: &'a MigrationConfig,
    rec: MigrationRecord,
}

impl Tracker<'_> {
    fn save(&self) -> Result<(), MigrationError> {
        match &self.cfg.journal {
            Some(j) => j.save(&self.rec),
            None => Ok(()),
        }
    }

    /// Record `p` durably, then die if `p` is the configured crash point.
    fn phase(&mut self, p: MigrationPhase) -> Result<(), MigrationError> {
        self.rec.phase = p;
        self.save()?;
        if p != MigrationPhase::Copying {
            self.crash_point(p)?;
        }
        Ok(())
    }

    fn crash_point(&self, p: MigrationPhase) -> Result<(), MigrationError> {
        if self.cfg.crash_after == Some(p) {
            return Err(MigrationError::Crashed(p));
        }
        Ok(())
    }

    fn clear(&self) {
        if let Some(j) = &self.cfg.journal {
            let _ = j.clear(self.rec.shard_id);
        }
    }
}

async fn run_migration(
    control: &dyn MapControl,
    r: &ShardRange,
    src: &Arc<dyn ShardNode>,
    dst: &Arc<dyn ShardNode>,
    to: &NodeId,
    t: &mut Tracker<'_>,
) -> Result<MigrationReport, MigrationError> {
    let cfg = t.cfg;
    let id = r.shard_id;
    dst.host_shard(id, r.lo, r.hi, false).await?;
    let mut seq = src.begin_capture(id).await?;
    t.phase(MigrationPhase::Copying)?;

    // 1. Snapshot copy (writes continue at the source).
    let mut copied = 0u64;
    let mut after = None;
    let mut chunks_done = 0u64;
    loop {
        let chunk = src.scan(id, after, cfg.chunk).await?;
        if chunk.is_empty() {
            break;
        }
        after = chunk.last().map(|(k, _)| *k);
        copied += chunk.len() as u64;
        let full = chunk.len() >= cfg.chunk;
        dst.apply_import(id, chunk).await?;
        t.rec.copied_keys = copied;
        t.rec.last_key = after;
        if chunks_done == 0 {
            t.save()?;
            t.crash_point(MigrationPhase::Copying)?;
        } else if chunks_done.is_multiple_of(64) {
            t.save()?;
        }
        chunks_done += 1;
        if !full {
            break;
        }
    }
    t.rec.copied_keys = copied;
    t.rec.last_key = after;

    // 2. Stream changes until the tail is small.
    t.phase(MigrationPhase::Streaming)?;
    let mut streamed = 0u64;
    for _ in 0..cfg.max_rounds {
        let ch = src.changes_since(id, seq, cfg.batch).await?;
        let n = ch.len();
        if n == 0 {
            break;
        }
        seq = ch.last().map(|c| c.seq).unwrap_or(seq);
        streamed += n as u64;
        dst.apply_changes(id, ch).await?;
        if n <= cfg.freeze_threshold {
            break;
        }
    }

    // 3. Brief write freeze, final catch-up, map switch, release.
    let t0 = Instant::now();
    src.freeze(id).await?;
    t.rec.streamed_changes = streamed;
    t.phase(MigrationPhase::Frozen)?;
    loop {
        let ch = src.changes_since(id, seq, cfg.batch).await?;
        let n = ch.len();
        if n == 0 {
            break;
        }
        seq = ch.last().map(|c| c.seq).unwrap_or(seq);
        streamed += n as u64;
        dst.apply_changes(id, ch).await?;
    }
    t.rec.streamed_changes = streamed;
    dst.activate(id).await?;
    t.phase(MigrationPhase::Activated)?;
    control
        .commit(ShardCmd::Move {
            shard_id: id,
            from: r.owner.clone(),
            to: to.clone(),
        })
        .await?;
    t.phase(MigrationPhase::Switched)?;
    // Past this point the map says `to` owns the shard; a failing release of
    // an unreachable source only leaves garbage there (it rejects nothing
    // it still serves, because clients now route to `to`). The journal entry
    // stays until the release succeeded, so recovery finishes the cleanup.
    if src.release(id).await.is_ok() {
        t.clear();
    }
    let cutover = t0.elapsed();
    Ok(MigrationReport {
        shard_id: id,
        from: r.owner.clone(),
        to: to.clone(),
        copied_keys: copied,
        streamed_changes: streamed,
        cutover,
    })
}

#[derive(Debug, Clone)]
pub struct DrainReport {
    pub node: NodeId,
    pub moved: Vec<MigrationReport>,
}

/// Pick the eligible node owning the fewest shards (ties: lowest id).
pub fn pick_target(table: &ShardTable, candidates: &[NodeId], exclude: &[NodeId]) -> Vec<NodeId> {
    let mut v: Vec<NodeId> = candidates
        .iter()
        .filter(|n| {
            !exclude.contains(n) && !table.draining.contains(n) && !table.drained.contains(n)
        })
        .cloned()
        .collect();
    v.sort_by_key(|n| (table.shards_of(n).len(), n.0));
    v
}

/// Migrate every shard off `node`, then mark it drained.
pub async fn drain_node(
    control: &dyn MapControl,
    nodes: &Nodes,
    node: &NodeId,
    cfg: &MigrationConfig,
) -> Result<DrainReport, MigrationError> {
    control
        .commit(ShardCmd::SetDraining {
            node: node.clone(),
            on: true,
        })
        .await?;
    let mut moved = Vec::new();
    loop {
        let table = control.table();
        let Some(shard) = table.shards_of(node).into_iter().next() else {
            break;
        };
        let mut done = false;
        let mut last_err = None;
        for target in pick_target(&table, &nodes.ids(), std::slice::from_ref(node)) {
            match migrate_shard(control, nodes, shard.shard_id, &target, cfg).await {
                Ok(rep) => {
                    moved.push(rep);
                    done = true;
                    break;
                }
                Err(e) => last_err = Some(e),
            }
        }
        if !done {
            return Err(last_err
                .unwrap_or_else(|| MigrationError::Rejected("no eligible drain target".into())));
        }
    }
    control
        .commit(ShardCmd::MarkDrained { node: node.clone() })
        .await?;
    Ok(DrainReport {
        node: node.clone(),
        moved,
    })
}

// ---------------------------------------------------------------------------
// Admin API (what `vvctl shard` talks to)
// ---------------------------------------------------------------------------

/// Request handler for the `/shards*` control endpoints. A node's HTTP
/// control server forwards `(method, path, body)` here and writes back the
/// returned `(status, json)`.
pub struct ShardAdmin {
    pub control: Arc<dyn MapControl>,
    pub nodes: Nodes,
    pub mcfg: MigrationConfig,
    pub rebalancer: Arc<crate::rebalance::Rebalancer>,
    collector: crate::rebalance::MetricsCollector,
}

#[derive(Deserialize)]
struct SplitReq {
    shard_id: u64,
    at: Option<u64>,
}
#[derive(Deserialize)]
struct MoveReq {
    shard_id: u64,
    to: uuid::Uuid,
}
#[derive(Deserialize)]
struct DrainReq {
    node: uuid::Uuid,
}

impl ShardAdmin {
    pub fn new(
        control: Arc<dyn MapControl>,
        nodes: Nodes,
        mcfg: MigrationConfig,
        rebalancer: Arc<crate::rebalance::Rebalancer>,
    ) -> Self {
        ShardAdmin {
            control,
            nodes,
            mcfg,
            rebalancer,
            collector: Default::default(),
        }
    }

    fn err(e: MigrationError) -> (u16, String) {
        let code = match &e {
            MigrationError::Rejected(_) | MigrationError::Aborted(_) => 409,
            MigrationError::NotFound(_) => 404,
            _ => 500,
        };
        (code, e.to_string())
    }

    pub async fn handle(&self, method: &str, path: &str, body: &str) -> (u16, String) {
        let bad = |e: serde_json::Error| (400u16, format!("bad request: {e}"));
        match (method, path) {
            ("GET", "/shards") => (
                200,
                serde_json::to_string(&self.control.table()).unwrap_or_default(),
            ),
            ("POST", "/shards/split") => {
                let req: SplitReq = match serde_json::from_str(body) {
                    Ok(r) => r,
                    Err(e) => return bad(e),
                };
                match split_shard(self.control.as_ref(), &self.nodes, req.shard_id, req.at).await {
                    Ok((lo, hi)) => (200, serde_json::json!({ "lo": lo, "hi": hi }).to_string()),
                    Err(e) => Self::err(e),
                }
            }
            ("POST", "/shards/move") => {
                let req: MoveReq = match serde_json::from_str(body) {
                    Ok(r) => r,
                    Err(e) => return bad(e),
                };
                match migrate_shard(
                    self.control.as_ref(),
                    &self.nodes,
                    req.shard_id,
                    &NodeId(req.to),
                    &self.mcfg,
                )
                .await
                {
                    Ok(r) => (
                        200,
                        serde_json::json!({
                            "shard_id": r.shard_id, "copied_keys": r.copied_keys,
                            "streamed_changes": r.streamed_changes,
                            "cutover_ms": r.cutover.as_millis() as u64,
                        })
                        .to_string(),
                    ),
                    Err(e) => Self::err(e),
                }
            }
            ("POST", "/shards/drain") => {
                let req: DrainReq = match serde_json::from_str(body) {
                    Ok(r) => r,
                    Err(e) => return bad(e),
                };
                match drain_node(
                    self.control.as_ref(),
                    &self.nodes,
                    &NodeId(req.node),
                    &self.mcfg,
                )
                .await
                {
                    Ok(r) => (
                        200,
                        serde_json::json!({ "drained": r.node, "shards_moved": r.moved.len() })
                            .to_string(),
                    ),
                    Err(e) => Self::err(e),
                }
            }
            ("POST", "/shards/rebalance") => {
                if !self.rebalancer.cfg.enabled {
                    return (409, "rebalancer is disabled (off by default)".into());
                }
                let metrics = self
                    .collector
                    .collect(&self.control.table(), &self.nodes)
                    .await;
                let r = self
                    .rebalancer
                    .run_round(
                        self.control.clone(),
                        self.nodes.clone(),
                        metrics,
                        self.mcfg.clone(),
                    )
                    .await;
                (
                    200,
                    serde_json::json!({
                        "splits": r.splits, "moves": r.moves.len(),
                        "rate_limited": r.rate_limited, "failures": r.failures,
                    })
                    .to_string(),
                )
            }
            _ => (404, "unknown shard endpoint".into()),
        }
    }
}
