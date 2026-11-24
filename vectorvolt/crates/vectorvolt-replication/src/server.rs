//! [`ReplicationServer`] / [`MultiReplicationServer`] — the leader side of
//! WAL-shipping replication.
//!
//! For every follower connection the server reads the follower's last applied
//! LSN, decides between streaming and a snapshot, ships committed WAL frames
//! (CRC-framed, still encrypted when the table is), and records the follower's
//! acks so writers can wait for them according to the configured mode.
//!
//! A snapshot is taken at a *consistent point*: the engine returns the WAL tip
//! together with the rows as of exactly that sequence
//! ([`TableEngine::consistent_snapshot`]), and streaming resumes right after
//! that tip, so snapshot + stream applies every mutation exactly once.
//!
//! The multi-table server serves every table of a [`TableRegistry`]: followers
//! open one catalog connection (table list, create/drop) plus one stream per
//! table, each with its own LSN position.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::{Condvar, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use vectorvolt_engine::{TableEngine, TableRegistry};

use crate::config::{ReplicationConfig, ReplicationMode, ReplicationRole, RoleCell};
use crate::error::{ReplResult, ReplicationError};
use crate::lineage::LineageStore;
use crate::metrics::ReplicationMetrics;
use crate::proto::{read_message, write_message, CatalogEntry, Message, PROTO_VERSION};
use crate::tls::{Conn, ConnRead, ConnWrite, ReplicationTlsServer};
use crate::wal_tail::{wal_bounds, WalBounds, WalTail};

/// File in a replicated table's directory holding its incarnation id.
pub(crate) const TID_FILE: &str = "replication.tid";

/// Per-follower acknowledgement state.
struct Slot {
    addr: SocketAddr,
    applied: AtomicU64,
    durable: AtomicU64,
    /// ms (server epoch) of the last ack that reached the leader tip.
    caught_up_ms: AtomicU64,
    /// Caught up to the leader tip it had when it connected; only in-sync
    /// followers count towards the configured replica count.
    in_sync: AtomicBool,
    /// Leader tip when the follower connected (or the snapshot's tip).
    join_tip: u64,
}

/// Snapshot of one connected follower, for lag reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowerInfo {
    pub id: u64,
    pub addr: SocketAddr,
    pub applied_lsn: u64,
    pub durable_lsn: u64,
    pub lag_lsn: u64,
    /// Milliseconds since this follower last had everything the leader had.
    pub lag_ms: u64,
}

/// Stop switch for an accept loop.
struct Stop {
    flag: AtomicBool,
    notify: Notify,
}

impl Stop {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            flag: AtomicBool::new(false),
            notify: Notify::new(),
        })
    }
    fn fire(&self) {
        self.flag.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }
    fn is_set(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }
}

/// State of one replicated table.
struct Inner {
    engine: Arc<TableEngine>,
    wal_dir: PathBuf,
    cfg: ReplicationConfig,
    role: RoleCell,
    metrics: Arc<ReplicationMetrics>,
    followers: Mutex<HashMap<u64, Arc<Slot>>>,
    next_id: AtomicU64,
    /// Highest WAL sequence seen on disk.
    tip: AtomicU64,
    shutdown: AtomicBool,
    wal_notify: Notify,
    ack_notify: Notify,
    ack_lock: Mutex<()>,
    ack_cv: Condvar,
    epoch: Instant,
    /// Follower acks a write needs (0 in async mode).
    needed: usize,
    /// Whether the mode counts durable (rather than applied) acks.
    durable_mode: bool,
    /// Highest LSN that `needed` followers have acknowledged (in the mode's
    /// sense). Recomputed on every ack / join / leave, so a waiter's check is
    /// one atomic load.
    quorum: AtomicU64,
    /// Fencing epoch this server leads (0 outside failover mode).
    fence_epoch: u64,
    /// Translation table for followers coming from an older epoch.
    lineage: Option<Arc<LineageStore>>,
    /// Currently running below the configured replica count.
    degraded: AtomicBool,
}

impl Inner {
    fn new(
        engine: Arc<TableEngine>,
        wal_dir: PathBuf,
        cfg: ReplicationConfig,
        role: RoleCell,
        epoch: u64,
        lineage: Option<Arc<LineageStore>>,
    ) -> ReplResult<Arc<Self>> {
        let tip = wal_bounds(&wal_dir)?.tip;
        let needed = cfg.required_acks();
        let durable_mode = cfg.mode == ReplicationMode::Sync;
        let inner = Arc::new(Self {
            engine,
            wal_dir,
            cfg,
            role,
            metrics: Arc::new(ReplicationMetrics::default()),
            followers: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            tip: AtomicU64::new(tip),
            shutdown: AtomicBool::new(false),
            wal_notify: Notify::new(),
            ack_notify: Notify::new(),
            ack_lock: Mutex::new(()),
            ack_cv: Condvar::new(),
            epoch: Instant::now(),
            needed,
            durable_mode,
            quorum: AtomicU64::new(0),
            fence_epoch: epoch,
            lineage,
            degraded: AtomicBool::new(false),
        });
        inner.recompute_quorum();
        Ok(inner)
    }

    fn in_sync_count(&self) -> usize {
        self.followers
            .lock()
            .values()
            .filter(|s| s.in_sync.load(Ordering::Acquire))
            .count()
    }

    /// `(acks a write needs right now, running degraded)`. The configured
    /// factor (`needed`) never changes; only when `degraded_ok` is set and too
    /// few followers are in sync does a write need fewer acks.
    fn effective(&self) -> (usize, bool) {
        if self.needed == 0 {
            return (0, false);
        }
        let in_sync = self.in_sync_count();
        if in_sync >= self.needed {
            (self.needed, false)
        } else if self.cfg.degraded_ok {
            (in_sync, true)
        } else {
            (self.needed, false)
        }
    }

    fn set_degraded(&self, d: bool, in_sync: usize) {
        if self.degraded.swap(d, Ordering::AcqRel) == d {
            return;
        }
        self.metrics.degraded.store(d as u64, Ordering::Relaxed);
        if d {
            tracing::warn!(
                needed = self.needed,
                in_sync,
                "replication DEGRADED: {in_sync} of {} required in-sync follower(s); \
                 writes are acknowledged with fewer replicas until followers catch up",
                self.needed
            );
        } else {
            tracing::info!(
                in_sync,
                "replication restored to the configured replica count"
            );
        }
    }

    /// Reject a write *before* applying it when too few in-sync followers are
    /// alive and degraded writes are not enabled.
    fn check_ready(&self) -> ReplResult<()> {
        if self.needed == 0 || self.cfg.degraded_ok {
            return Ok(());
        }
        let in_sync = self.in_sync_count();
        if in_sync >= self.needed {
            Ok(())
        } else {
            Err(ReplicationError::InsufficientReplicas {
                mode: self.cfg.mode.to_string(),
                needed: self.needed,
                in_sync,
            })
        }
    }

    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    fn slot_lsn(s: &Slot, durable: bool) -> u64 {
        if durable {
            s.durable.load(Ordering::Acquire)
        } else {
            s.applied.load(Ordering::Acquire)
        }
    }

    fn acks_for(&self, lsn: u64, durable: bool) -> usize {
        self.followers
            .lock()
            .values()
            .filter(|s| Self::slot_lsn(s, durable) >= lsn)
            .count()
    }

    /// Recompute [`quorum`](Self::quorum) and wake every waiter.
    fn recompute_quorum(&self) {
        if self.needed == 0 {
            return;
        }
        let (eff, degraded) = self.effective();
        self.set_degraded(degraded, self.in_sync_count());
        let q = if eff == 0 {
            u64::MAX
        } else {
            let f = self.followers.lock();
            let d = self.durable_mode;
            let mut best = 0;
            for a in f.values() {
                let v = Self::slot_lsn(a, d);
                if v > best && f.values().filter(|b| Self::slot_lsn(b, d) >= v).count() >= eff {
                    best = v;
                }
            }
            best
        };
        self.quorum.store(q, Ordering::Release);
        {
            // Taking the lock orders this notify after any blocking waiter's
            // check-then-wait.
            let _g = self.ack_lock.lock();
            self.ack_cv.notify_all();
        }
        self.ack_notify.notify_waiters();
    }

    fn check_role(&self) -> ReplResult<()> {
        match self.role.get() {
            ReplicationRole::Follower => Err(ReplicationError::NotLeader),
            _ => Ok(()),
        }
    }

    fn timeout_err(&self, lsn: u64) -> ReplicationError {
        self.metrics.ack_timeouts.fetch_add(1, Ordering::Relaxed);
        ReplicationError::AckTimeout {
            lsn,
            needed: self.needed,
            got: self.acks_for(lsn, self.durable_mode),
            timeout: self.cfg.ack_timeout,
        }
    }

    fn followers_info(&self) -> Vec<FollowerInfo> {
        let tip = self.tip.load(Ordering::Acquire);
        let now = self.now_ms();
        let mut v: Vec<FollowerInfo> = self
            .followers
            .lock()
            .iter()
            .map(|(id, s)| {
                let applied = s.applied.load(Ordering::Acquire);
                FollowerInfo {
                    id: *id,
                    addr: s.addr,
                    applied_lsn: applied,
                    durable_lsn: s.durable.load(Ordering::Acquire),
                    lag_lsn: tip.saturating_sub(applied),
                    lag_ms: if applied >= tip {
                        0
                    } else {
                        now.saturating_sub(s.caught_up_ms.load(Ordering::Acquire))
                    },
                }
            })
            .collect();
        v.sort_by_key(|f| f.id);
        v
    }

    /// Wait for the replication mode's ack requirement on `lsn`.
    ///
    /// Async mode and an already-acknowledged `lsn` return after one or two
    /// atomic loads, with no allocation, lock or timer. Otherwise the caller
    /// parks on `ack_notify` (registered *before* re-checking, so no ack is
    /// missed) until an ack arrives or `ack_timeout` passes.
    async fn wait_replicated(&self, lsn: u64) -> ReplResult<()> {
        if self.needed == 0 {
            return Ok(());
        }
        self.check_role()?;
        if self.degraded.load(Ordering::Acquire) {
            self.metrics.degraded_writes.fetch_add(1, Ordering::Relaxed);
        }
        if self.quorum.load(Ordering::Acquire) >= lsn {
            return Ok(());
        }
        let deadline = Instant::now() + self.cfg.ack_timeout;
        self.wal_notify.notify_waiters();
        loop {
            let notified = self.ack_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.quorum.load(Ordering::Acquire) >= lsn {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(self.timeout_err(lsn));
            }
            let wait = (deadline - now).min(Duration::from_millis(100));
            tokio::select! {
                _ = &mut notified => {}
                _ = tokio::time::sleep(wait) => {}
            }
            self.check_role()?;
        }
    }

    fn wait_replicated_blocking(&self, lsn: u64) -> ReplResult<()> {
        if self.needed == 0 {
            return Ok(());
        }
        self.check_role()?;
        if self.degraded.load(Ordering::Acquire) {
            self.metrics.degraded_writes.fetch_add(1, Ordering::Relaxed);
        }
        if self.quorum.load(Ordering::Acquire) >= lsn {
            return Ok(());
        }
        let deadline = Instant::now() + self.cfg.ack_timeout;
        self.wal_notify.notify_waiters();
        let mut guard = self.ack_lock.lock();
        loop {
            if self.quorum.load(Ordering::Acquire) >= lsn {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(self.timeout_err(lsn));
            }
            let wait = (deadline - now).min(Duration::from_millis(100));
            self.ack_cv.wait_for(&mut guard, wait);
            self.check_role()?;
        }
    }
}

// ── single-table server ──────────────────────────────────────────────────────

/// Leader-side replication endpoint for one table.
pub struct ReplicationServer {
    inner: Arc<Inner>,
    stop: Arc<Stop>,
    local_addr: SocketAddr,
    accept: Mutex<Option<JoinHandle<()>>>,
}

impl ReplicationServer {
    /// Bind `listen` and start accepting followers on the current tokio runtime.
    ///
    /// `wal_dir` is the table's WAL directory (`<data>/wal`). Frames are
    /// shipped exactly as stored, so an encrypted table needs no key on the
    /// leader side; the follower decrypts with the same table key.
    pub async fn start(
        engine: Arc<TableEngine>,
        wal_dir: impl Into<PathBuf>,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
    ) -> ReplResult<Arc<Self>> {
        Self::start_tls(engine, wal_dir, cfg, role, listen, None).await
    }

    /// [`start`](Self::start) with optional TLS / mTLS. With `Some(tls)` the
    /// listener only accepts TLS connections.
    pub async fn start_tls(
        engine: Arc<TableEngine>,
        wal_dir: impl Into<PathBuf>,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
        tls: Option<ReplicationTlsServer>,
    ) -> ReplResult<Arc<Self>> {
        Self::start_inner(engine, wal_dir.into(), cfg, role, listen, tls, 0, None).await
    }

    /// Failover-mode server: every message carries `epoch`, and followers
    /// arriving from an older epoch are matched to a common point through
    /// `lineage` (this node's record of the primaries it followed or led).
    #[allow(clippy::too_many_arguments)]
    pub async fn start_epoch(
        engine: Arc<TableEngine>,
        wal_dir: impl Into<PathBuf>,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
        epoch: u64,
        lineage: Arc<LineageStore>,
    ) -> ReplResult<Arc<Self>> {
        Self::start_inner(
            engine,
            wal_dir.into(),
            cfg,
            role,
            listen,
            None,
            epoch,
            Some(lineage),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn start_inner(
        engine: Arc<TableEngine>,
        wal_dir: PathBuf,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
        tls: Option<ReplicationTlsServer>,
        epoch: u64,
        lineage: Option<Arc<LineageStore>>,
    ) -> ReplResult<Arc<Self>> {
        cfg.validate()?;
        let inner = Inner::new(engine, wal_dir, cfg, role, epoch, lineage)?;
        let listener = TcpListener::bind(listen).await?;
        let local_addr = listener.local_addr()?;
        let stop = Stop::new();
        let handle = tokio::spawn(accept_loop(
            listener,
            tls,
            Target::Single(Arc::clone(&inner)),
            Arc::clone(&stop),
        ));
        Ok(Arc::new(Self {
            inner,
            stop,
            local_addr,
            accept: Mutex::new(Some(handle)),
        }))
    }

    /// Address actually bound (useful with port 0).
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn metrics(&self) -> Arc<ReplicationMetrics> {
        Arc::clone(&self.inner.metrics)
    }

    pub fn role(&self) -> &RoleCell {
        &self.inner.role
    }

    pub fn config(&self) -> &ReplicationConfig {
        &self.inner.cfg
    }

    /// Connected followers with their ack positions and lag.
    pub fn followers(&self) -> Vec<FollowerInfo> {
        self.inner.followers_info()
    }

    /// Number of followers currently connected.
    pub fn follower_count(&self) -> usize {
        self.inner.followers.lock().len()
    }

    /// Followers that are caught up and so count towards the replica count.
    pub fn in_sync_followers(&self) -> usize {
        self.inner.in_sync_count()
    }

    /// True while writes are acknowledged by fewer followers than configured
    /// (only possible with `degraded_ok`).
    pub fn is_degraded(&self) -> bool {
        self.inner.degraded.load(Ordering::Acquire)
    }

    /// The fencing epoch this server leads.
    pub fn epoch(&self) -> u64 {
        self.inner.fence_epoch
    }

    /// `Err(InsufficientReplicas)` when a write now could not meet the
    /// replication mode and degraded writes are off. Call *before* applying a
    /// write so a refused write leaves no trace.
    pub fn check_ready(&self) -> ReplResult<()> {
        self.inner.check_ready()
    }

    /// Wake streaming connections now instead of at the next poll. Call after
    /// a local engine write to cut replication latency.
    pub fn notify_write(&self) {
        self.inner.wal_notify.notify_waiters();
    }

    /// Wait until the write that produced WAL sequence `lsn` is replicated as
    /// the configured mode requires:
    ///
    /// - `Async`: returns immediately (no allocation, no lock).
    /// - `SemiSync`: one follower has applied it.
    /// - `Sync`: `replication_factor - 1` followers have durably applied it.
    ///
    /// Cheap enough to call on every write: an already-acknowledged `lsn` is
    /// one atomic load, and a pending one parks on a `Notify` that the ack
    /// reader fires. Returns [`ReplicationError::AckTimeout`] if the acks do
    /// not arrive within `ack_timeout` — never a silent success.
    pub async fn wait_for_replication(&self, lsn: u64) -> ReplResult<()> {
        self.inner.wait_replicated(lsn).await
    }

    /// Blocking variant of [`wait_for_replication`](Self::wait_for_replication)
    /// for synchronous callers (must not be called from an async task).
    pub fn wait_for_replication_blocking(&self, lsn: u64) -> ReplResult<()> {
        self.inner.wait_replicated_blocking(lsn)
    }

    /// Insert on the leader engine, then wait per the replication mode.
    /// Returns the WAL sequence of the write.
    pub async fn insert_replicated(&self, pk: u64, values: Vec<u64>) -> ReplResult<u64> {
        self.inner.check_role()?;
        self.inner.check_ready()?;
        let tip = self.inner.engine.insert(pk, values)?;
        self.wait_for_replication(tip.sequence).await?;
        Ok(tip.sequence)
    }

    /// Delete on the leader engine, then wait per the replication mode.
    pub async fn delete_replicated(&self, pk: u64) -> ReplResult<u64> {
        self.inner.check_role()?;
        self.inner.check_ready()?;
        let tip = self.inner.engine.delete(pk)?;
        self.wait_for_replication(tip.sequence).await?;
        Ok(tip.sequence)
    }

    /// Stop accepting and drop every follower stream.
    pub fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        self.inner.wal_notify.notify_waiters();
        self.stop.fire();
        if let Some(h) = self.accept.lock().take() {
            h.abort();
        }
    }
}

impl Drop for ReplicationServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ── multi-table server ───────────────────────────────────────────────────────

struct CatalogCacheEntry {
    engine: Arc<TableEngine>,
    entry: CatalogEntry,
}

struct MultiShared {
    registry: Arc<TableRegistry>,
    cfg: ReplicationConfig,
    role: RoleCell,
    tables: Mutex<HashMap<String, Arc<Inner>>>,
    catalog_cache: Mutex<HashMap<String, CatalogCacheEntry>>,
    stop: Arc<Stop>,
}

static TID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn fresh_tid() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    (nanos ^ (TID_COUNTER.fetch_add(1, Ordering::Relaxed) << 48)).max(1)
}

pub(crate) fn read_tid(dir: &std::path::Path) -> Option<u64> {
    let b = std::fs::read(dir.join(TID_FILE)).ok()?;
    <[u8; 8]>::try_from(b.as_slice())
        .ok()
        .map(u64::from_le_bytes)
}

pub(crate) fn write_tid(dir: &std::path::Path, id: u64) -> std::io::Result<()> {
    std::fs::write(dir.join(TID_FILE), id.to_le_bytes())
}

impl MultiShared {
    /// The replication state for `name`, created on first use. A table that
    /// was dropped and re-created since is replaced.
    fn table(&self, name: &str) -> ReplResult<Arc<Inner>> {
        let engine = self.registry.get(name)?;
        let mut tables = self.tables.lock();
        if let Some(t) = tables.get(name) {
            if Arc::ptr_eq(&t.engine, &engine) {
                return Ok(Arc::clone(t));
            }
            t.shutdown.store(true, Ordering::Release);
            t.wal_notify.notify_waiters();
        }
        let wal_dir = self.registry.base_dir().join(name).join("wal");
        let inner = Inner::new(
            engine,
            wal_dir,
            self.cfg.clone(),
            self.role.clone(),
            0,
            None,
        )?;
        tables.insert(name.to_owned(), Arc::clone(&inner));
        Ok(inner)
    }

    /// Forget tables that no longer exist, ending their streams.
    fn sync_dropped(&self) {
        let live: std::collections::HashSet<String> =
            self.registry.list_tables().into_iter().collect();
        self.tables.lock().retain(|name, t| {
            let keep = live.contains(name);
            if !keep {
                t.shutdown.store(true, Ordering::Release);
                t.wal_notify.notify_waiters();
            }
            keep
        });
        let base = self.registry.base_dir().to_owned();
        self.catalog_cache.lock().retain(|name, _| {
            let keep = live.contains(name);
            if !keep {
                let _ = std::fs::remove_file(base.join(name).join(TID_FILE));
            }
            keep
        });
    }

    /// The current table list (sorted), with each table's incarnation id.
    fn catalog(&self) -> Vec<CatalogEntry> {
        self.sync_dropped();
        let mut cache = self.catalog_cache.lock();
        let mut out = Vec::new();
        for name in self.registry.list_tables() {
            let Ok(engine) = self.registry.get(&name) else {
                continue;
            };
            if let Some(c) = cache.get(&name) {
                if Arc::ptr_eq(&c.engine, &engine) {
                    out.push(c.entry.clone());
                    continue;
                }
            }
            let Ok(schema) = self.registry.read_schema(&name) else {
                continue;
            };
            let dir = self.registry.base_dir().join(&name);
            let recreated = cache.contains_key(&name);
            let id = match (recreated, read_tid(&dir)) {
                (false, Some(id)) => id,
                _ => {
                    let id = fresh_tid();
                    let _ = write_tid(&dir, id);
                    id
                }
            };
            let entry = CatalogEntry {
                name: name.clone(),
                schema: schema.to_text(),
                id,
            };
            out.push(entry.clone());
            cache.insert(name, CatalogCacheEntry { engine, entry });
        }
        out
    }
}

/// Leader-side replication endpoint for every table of a [`TableRegistry`].
///
/// Followers (see `MultiReplicationClient`) learn the table list over a catalog
/// connection — including tables created, dropped or re-created later — and
/// replicate each table over its own stream with its own LSN.
pub struct MultiReplicationServer {
    shared: Arc<MultiShared>,
    local_addr: SocketAddr,
    accept: Mutex<Option<JoinHandle<()>>>,
}

impl MultiReplicationServer {
    pub async fn start(
        registry: Arc<TableRegistry>,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
    ) -> ReplResult<Arc<Self>> {
        Self::start_tls(registry, cfg, role, listen, None).await
    }

    pub async fn start_tls(
        registry: Arc<TableRegistry>,
        cfg: ReplicationConfig,
        role: RoleCell,
        listen: SocketAddr,
        tls: Option<ReplicationTlsServer>,
    ) -> ReplResult<Arc<Self>> {
        cfg.validate()?;
        let listener = TcpListener::bind(listen).await?;
        let local_addr = listener.local_addr()?;
        let shared = Arc::new(MultiShared {
            registry,
            cfg,
            role,
            tables: Mutex::new(HashMap::new()),
            catalog_cache: Mutex::new(HashMap::new()),
            stop: Stop::new(),
        });
        let handle = tokio::spawn(accept_loop(
            listener,
            tls,
            Target::Multi(Arc::clone(&shared)),
            Arc::clone(&shared.stop),
        ));
        Ok(Arc::new(Self {
            shared,
            local_addr,
            accept: Mutex::new(Some(handle)),
        }))
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn role(&self) -> &RoleCell {
        &self.shared.role
    }

    pub fn config(&self) -> &ReplicationConfig {
        &self.shared.cfg
    }

    /// Metrics of one table's replication state.
    pub fn table_metrics(&self, table: &str) -> ReplResult<Arc<ReplicationMetrics>> {
        Ok(Arc::clone(&self.shared.table(table)?.metrics))
    }

    /// Followers streaming `table`.
    pub fn followers(&self, table: &str) -> ReplResult<Vec<FollowerInfo>> {
        Ok(self.shared.table(table)?.followers_info())
    }

    /// Wake every table's streams now instead of at the next poll.
    pub fn notify_write(&self) {
        for t in self.shared.tables.lock().values() {
            t.wal_notify.notify_waiters();
        }
    }

    /// [`ReplicationServer::wait_for_replication`] for one table.
    pub async fn wait_for_replication(&self, table: &str, lsn: u64) -> ReplResult<()> {
        self.shared.table(table)?.wait_replicated(lsn).await
    }

    /// Blocking variant (not for async tasks).
    pub fn wait_for_replication_blocking(&self, table: &str, lsn: u64) -> ReplResult<()> {
        self.shared.table(table)?.wait_replicated_blocking(lsn)
    }

    /// Insert into `table` on the leader, then wait per the replication mode.
    pub async fn insert_replicated(
        &self,
        table: &str,
        pk: u64,
        values: Vec<u64>,
    ) -> ReplResult<u64> {
        let t = self.shared.table(table)?;
        t.check_role()?;
        t.check_ready()?;
        let tip = t.engine.insert(pk, values)?;
        t.wait_replicated(tip.sequence).await?;
        Ok(tip.sequence)
    }

    /// Stop accepting and drop every stream.
    pub fn shutdown(&self) {
        for t in self.shared.tables.lock().values() {
            t.shutdown.store(true, Ordering::Release);
            t.wal_notify.notify_waiters();
        }
        self.shared.stop.fire();
        if let Some(h) = self.accept.lock().take() {
            h.abort();
        }
    }
}

impl Drop for MultiReplicationServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ── accept / route ───────────────────────────────────────────────────────────

#[derive(Clone)]
enum Target {
    Single(Arc<Inner>),
    Multi(Arc<MultiShared>),
}

async fn accept_loop(
    listener: TcpListener,
    tls: Option<ReplicationTlsServer>,
    target: Target,
    stop: Arc<Stop>,
) {
    loop {
        let accepted = tokio::select! {
            r = listener.accept() => r,
            _ = stop.notify.notified() => break,
        };
        if stop.is_set() {
            break;
        }
        match accepted {
            Ok((stream, peer)) => {
                let target = target.clone();
                let tls = tls.clone();
                tokio::spawn(async move {
                    if let Err(e) = serve_conn(target, tls, stream, peer).await {
                        tracing::debug!(%peer, "replication stream ended: {e}");
                    }
                });
            }
            Err(e) => {
                tracing::warn!("replication accept failed: {e}");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

async fn send(inner: &Inner, w: &mut ConnWrite, msg: &Message) -> ReplResult<()> {
    let n = write_message(w, msg).await?;
    inner
        .metrics
        .bytes_sent
        .fetch_add(n as u64, Ordering::Relaxed);
    Ok(())
}

async fn reject(w: &mut ConnWrite, why: String) -> ReplicationError {
    let _ = write_message(w, &Message::Error(why.clone())).await;
    ReplicationError::Protocol(why)
}

async fn serve_conn(
    target: Target,
    tls: Option<ReplicationTlsServer>,
    stream: TcpStream,
    peer: SocketAddr,
) -> ReplResult<()> {
    stream.set_nodelay(true)?;
    let conn: Conn = match tls {
        Some(t) => Box::new(
            tokio::time::timeout(Duration::from_secs(5), t.acceptor.accept(stream))
                .await
                .map_err(|_| ReplicationError::Protocol("tls handshake timeout".into()))??,
        ),
        None => Box::new(stream),
    };
    let (mut rd, mut wr) = tokio::io::split(conn);
    let (hello, _) = tokio::time::timeout(Duration::from_secs(5), read_message(&mut rd))
        .await
        .map_err(|_| ReplicationError::Protocol("hello timeout".into()))??;
    // Older hellos are parsed (not treated as corruption) so they can be
    // answered with a clear error; the stream itself is v2 only.
    let bad_version = |v: u8| -> Option<String> {
        match v {
            PROTO_VERSION => None,
            1 => Some(
                "protocol v1 follower: this leader streams v2 (per-frame epochs); \
                 upgrade the follower"
                    .into(),
            ),
            _ => Some(format!("unsupported protocol version {v}")),
        }
    };
    match (hello, target) {
        (
            Message::Hello {
                version,
                applied_lsn,
                epoch,
                history,
            },
            Target::Single(inner),
        ) => {
            if let Some(why) = bad_version(version) {
                return Err(reject(&mut wr, why).await);
            }
            if epoch > inner.fence_epoch {
                return Err(reject(
                    &mut wr,
                    format!(
                        "stale primary: follower is at epoch {epoch}, this leader at {}",
                        inner.fence_epoch
                    ),
                )
                .await);
            }
            let hello = HelloInfo {
                applied: applied_lsn,
                epoch,
                history,
            };
            serve_table(inner, rd, wr, peer, hello).await
        }
        (
            Message::HelloTable {
                version,
                table,
                applied_lsn,
            },
            Target::Multi(m),
        ) => {
            if let Some(why) = bad_version(version) {
                return Err(reject(&mut wr, why).await);
            }
            let inner = match m.table(&table) {
                Ok(i) => i,
                Err(e) => return Err(reject(&mut wr, format!("table {table:?}: {e}")).await),
            };
            let hello = HelloInfo {
                applied: applied_lsn,
                epoch: 0,
                history: Vec::new(),
            };
            serve_table(inner, rd, wr, peer, hello).await
        }
        (Message::CatalogHello { version }, Target::Multi(m)) => {
            if let Some(why) = bad_version(version) {
                return Err(reject(&mut wr, why).await);
            }
            serve_catalog(m, rd, wr).await
        }
        _ => Err(reject(&mut wr, "unexpected hello for this leader".into()).await),
    }
}

async fn serve_catalog(m: Arc<MultiShared>, mut rd: ConnRead, mut wr: ConnWrite) -> ReplResult<()> {
    let dead = Arc::new(AtomicBool::new(false));
    let reader_dead = Arc::clone(&dead);
    let reader = tokio::spawn(async move {
        while read_message(&mut rd).await.is_ok() {}
        reader_dead.store(true, Ordering::Release);
    });
    let mut last: Option<Vec<CatalogEntry>> = None;
    let mut last_send = Instant::now();
    let result = loop {
        if m.stop.is_set() || dead.load(Ordering::Acquire) {
            break Ok(());
        }
        if m.role.get() == ReplicationRole::Follower {
            let _ = write_message(&mut wr, &Message::Error("leadership lost".into())).await;
            break Err(ReplicationError::NotLeader);
        }
        let shared = Arc::clone(&m);
        let cat = match tokio::task::spawn_blocking(move || shared.catalog()).await {
            Ok(c) => c,
            Err(e) => break Err(ReplicationError::Engine(e.to_string())),
        };
        let r = if last.as_ref() != Some(&cat) {
            last = Some(cat.clone());
            last_send = Instant::now();
            write_message(&mut wr, &Message::Catalog(cat)).await
        } else if last_send.elapsed() >= m.cfg.heartbeat_interval {
            last_send = Instant::now();
            write_message(&mut wr, &Message::Heartbeat { epoch: 0, tip: 0 }).await
        } else {
            Ok(0)
        };
        if let Err(e) = r {
            break Err(e);
        }
        tokio::time::sleep(m.cfg.poll_interval.max(Duration::from_millis(5))).await;
    };
    reader.abort();
    result
}

// ── per-table stream ─────────────────────────────────────────────────────────

struct SlotGuard {
    inner: Arc<Inner>,
    id: u64,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        let empty = {
            let mut f = self.inner.followers.lock();
            f.remove(&self.id);
            f.is_empty()
        };
        self.inner
            .metrics
            .connected
            .store((!empty) as u64, Ordering::Relaxed);
        self.inner.recompute_quorum();
    }
}

/// What a follower sent in its hello.
struct HelloInfo {
    applied: u64,
    epoch: u64,
    history: Vec<(u64, u64)>,
}

/// How to bring a follower level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    /// Stream from `start` (this server's WAL sequence).
    Stream {
        start: u64,
    },
    /// The follower holds a divergent tail: fix it, then stream from `start`.
    Truncate {
        start: u64,
        common_epoch: u64,
        common_lsn: u64,
    },
    Snapshot,
}

/// Decide between streaming, truncate-and-stream and a full snapshot.
///
/// Same epoch (or no lineage): the follower's LSN is in this server's LSN
/// space. Otherwise the history is matched like Raft's log matching: the newest
/// epoch both sides know gives the common point, which this node's lineage maps
/// onto its own WAL. A follower that applied more than the common point holds a
/// divergent tail. No usable common point means a snapshot.
fn plan(inner: &Inner, bounds: &WalBounds, h: &HelloInfo) -> Plan {
    let tip = bounds.tip;
    let floor = bounds.first_seq.map(|f| f.saturating_sub(1)).unwrap_or(0);
    let in_range = |l: u64| l <= tip && l >= floor;
    let Some(lineage) = inner
        .lineage
        .as_ref()
        .filter(|_| h.epoch != inner.fence_epoch)
    else {
        return if h.applied > tip || h.applied < floor {
            Plan::Snapshot
        } else {
            Plan::Stream { start: h.applied }
        };
    };
    let lin = lineage.lock();
    for &(e, k) in &h.history {
        if e >= inner.fence_epoch {
            continue;
        }
        if let Some(m) = lin.map(e, k, tip) {
            if !in_range(m.local) {
                return Plan::Snapshot;
            }
            let diverged = !(h.epoch == e && h.applied == m.common_lsn);
            return if diverged {
                Plan::Truncate {
                    start: m.local,
                    common_epoch: e,
                    common_lsn: m.common_lsn,
                }
            } else {
                Plan::Stream { start: m.local }
            };
        }
    }
    Plan::Snapshot
}

async fn send_rows(
    inner: &Inner,
    wr: &mut ConnWrite,
    rows: Vec<(u64, Vec<u64>)>,
    end_lsn: u64,
) -> ReplResult<()> {
    for chunk in rows.chunks(inner.cfg.snapshot_batch_rows) {
        send(inner, wr, &Message::SnapshotRows(chunk.to_vec())).await?;
    }
    send(
        inner,
        wr,
        &Message::SnapshotEnd {
            snapshot_lsn: end_lsn,
        },
    )
    .await
}

async fn serve_table(
    inner: Arc<Inner>,
    mut rd: ConnRead,
    mut wr: ConnWrite,
    peer: SocketAddr,
    hello: HelloInfo,
) -> ReplResult<()> {
    if inner.role.get() == ReplicationRole::Follower {
        let _ = send(&inner, &mut wr, &Message::Error("not the leader".into())).await;
        return Err(ReplicationError::NotLeader);
    }

    let wal_dir = inner.wal_dir.clone();
    let bounds = tokio::task::spawn_blocking(move || wal_bounds(&wal_dir))
        .await
        .map_err(|e| ReplicationError::Wal(e.to_string()))??;
    inner.tip.fetch_max(bounds.tip, Ordering::AcqRel);
    let plan = plan(&inner, &bounds, &hello);
    // A snapshot is taken at a consistent point (WAL tip + rows as of that
    // exact tip), and streaming resumes right after it.
    let (start_lsn, snapshot_rows) = if plan == Plan::Snapshot {
        let engine = Arc::clone(&inner.engine);
        let (tip, rows) = tokio::task::spawn_blocking(move || engine.consistent_snapshot())
            .await
            .map_err(|e| ReplicationError::Engine(e.to_string()))?;
        inner.tip.fetch_max(tip, Ordering::AcqRel);
        let rows: Vec<(u64, Vec<u64>)> = rows.into_iter().map(|r| (r.pk, r.values)).collect();
        (tip, Some(rows))
    } else {
        match plan {
            Plan::Stream { start } | Plan::Truncate { start, .. } => (start, None),
            Plan::Snapshot => unreachable!("handled above"),
        }
    };
    let resumed = matches!(plan, Plan::Stream { .. });

    let id = inner.next_id.fetch_add(1, Ordering::Relaxed);
    let join_tip = bounds.tip.max(start_lsn);
    let slot = Arc::new(Slot {
        addr: peer,
        applied: AtomicU64::new(if resumed { start_lsn } else { 0 }),
        durable: AtomicU64::new(if resumed { start_lsn } else { 0 }),
        caught_up_ms: AtomicU64::new(inner.now_ms()),
        in_sync: AtomicBool::new(resumed && start_lsn >= join_tip),
        join_tip,
    });
    inner.followers.lock().insert(id, Arc::clone(&slot));
    inner.metrics.connected.store(1, Ordering::Relaxed);
    let _guard = SlotGuard {
        inner: Arc::clone(&inner),
        id,
    };
    inner.recompute_quorum();

    let (truncate, common_epoch, common_lsn) = match plan {
        Plan::Truncate {
            common_epoch,
            common_lsn,
            ..
        } => (true, common_epoch, common_lsn),
        _ => (false, 0, 0),
    };
    send(
        &inner,
        &mut wr,
        &Message::Welcome {
            first_lsn: bounds.first_seq.unwrap_or(1),
            tip: bounds.tip.max(start_lsn),
            snapshot: plan == Plan::Snapshot,
            start_lsn,
            epoch: inner.fence_epoch,
            truncate,
            common_epoch,
            common_lsn,
        },
    )
    .await?;

    // Truncating handshake: the follower names the keys its divergent tail
    // touched; answer with this node's current rows for them (absent keys are
    // deleted by the follower), or fall back to a snapshot if it cannot.
    let mut start_lsn = start_lsn;
    let mut snapshot_rows = snapshot_rows;
    if truncate {
        let (msg, n) = tokio::time::timeout(Duration::from_secs(30), read_message(&mut rd))
            .await
            .map_err(|_| ReplicationError::Protocol("tail keys timeout".into()))??;
        inner
            .metrics
            .bytes_received
            .fetch_add(n as u64, Ordering::Relaxed);
        match msg {
            Message::TailKeys { ok: true, pks } => {
                let engine = Arc::clone(&inner.engine);
                let rows =
                    tokio::task::spawn_blocking(move || -> ReplResult<Vec<(u64, Vec<u64>)>> {
                        let mut out = Vec::new();
                        for pk in pks {
                            if let Some(r) = engine.get(pk)? {
                                out.push((r.pk, r.values));
                            }
                        }
                        Ok(out)
                    })
                    .await
                    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                send_rows(&inner, &mut wr, rows, start_lsn).await?;
            }
            Message::TailKeys { ok: false, .. } => {
                let engine = Arc::clone(&inner.engine);
                let (tip, rows) = tokio::task::spawn_blocking(move || engine.consistent_snapshot())
                    .await
                    .map_err(|e| ReplicationError::Engine(e.to_string()))?;
                inner.tip.fetch_max(tip, Ordering::AcqRel);
                snapshot_rows = Some(rows.into_iter().map(|r| (r.pk, r.values)).collect());
                start_lsn = tip;
            }
            _ => {
                return Err(reject(&mut wr, "expected tail keys".into()).await);
            }
        }
    }

    // Ack reader.
    let dead = Arc::new(AtomicBool::new(false));
    let ack_inner = Arc::clone(&inner);
    let ack_slot = Arc::clone(&slot);
    let ack_dead = Arc::clone(&dead);
    let ack_task = tokio::spawn(async move {
        loop {
            match read_message(&mut rd).await {
                Ok((
                    Message::Ack {
                        epoch,
                        applied_lsn,
                        durable_lsn,
                    },
                    n,
                )) => {
                    ack_inner
                        .metrics
                        .bytes_received
                        .fetch_add(n as u64, Ordering::Relaxed);
                    if epoch != ack_inner.fence_epoch {
                        // An ack from another epoch says nothing about this
                        // primary's log: it must never count towards a quorum.
                        ack_inner.metrics.stale_acks.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    ack_slot.applied.fetch_max(applied_lsn, Ordering::AcqRel);
                    ack_slot.durable.fetch_max(durable_lsn, Ordering::AcqRel);
                    if applied_lsn >= ack_slot.join_tip {
                        ack_slot.in_sync.store(true, Ordering::Release);
                    }
                    if applied_lsn >= ack_inner.tip.load(Ordering::Acquire) {
                        ack_slot
                            .caught_up_ms
                            .store(ack_inner.now_ms(), Ordering::Release);
                    }
                    ack_inner.recompute_quorum();
                }
                Ok(_) => {}
                Err(e) => {
                    if matches!(e, ReplicationError::Corrupt(_)) {
                        ack_inner
                            .metrics
                            .rejected_frames
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    break;
                }
            }
        }
        ack_dead.store(true, Ordering::Release);
        ack_inner.wal_notify.notify_waiters();
    });

    let result = stream_loop(&inner, &mut wr, &dead, snapshot_rows, start_lsn).await;
    ack_task.abort();
    result
}

async fn stream_loop(
    inner: &Arc<Inner>,
    wr: &mut ConnWrite,
    dead: &AtomicBool,
    snapshot_rows: Option<Vec<(u64, Vec<u64>)>>,
    start_lsn: u64,
) -> ReplResult<()> {
    if let Some(rows) = snapshot_rows {
        send_rows(inner, wr, rows, start_lsn).await?;
        inner.metrics.snapshots_sent.fetch_add(1, Ordering::Relaxed);
    }

    let wal_dir = inner.wal_dir.clone();
    let mut tail = tokio::task::spawn_blocking(move || WalTail::open(&wal_dir, start_lsn))
        .await
        .map_err(|e| ReplicationError::Wal(e.to_string()))??;
    let max = inner.cfg.batch_frames;
    let mut last_send = Instant::now();
    loop {
        if inner.shutdown.load(Ordering::Acquire) || dead.load(Ordering::Acquire) {
            return Ok(());
        }
        if inner.role.get() == ReplicationRole::Follower {
            let _ = send(inner, wr, &Message::Error("leadership lost".into())).await;
            return Err(ReplicationError::NotLeader);
        }
        let (t, batch) = tokio::task::spawn_blocking(move || {
            let r = tail.read_batch(max);
            (tail, r)
        })
        .await
        .map_err(|e| ReplicationError::Wal(e.to_string()))?;
        tail = t;
        let batch = batch?;
        if batch.is_empty() {
            if last_send.elapsed() >= inner.cfg.heartbeat_interval {
                let tip = inner.tip.load(Ordering::Acquire).max(tail.last_seq);
                send(
                    inner,
                    wr,
                    &Message::Heartbeat {
                        epoch: inner.fence_epoch,
                        tip,
                    },
                )
                .await?;
                last_send = Instant::now();
            }
            tokio::select! {
                _ = inner.wal_notify.notified() => {}
                _ = tokio::time::sleep(inner.cfg.poll_interval) => {}
            }
            continue;
        }
        let tip = inner
            .tip
            .fetch_max(tail.last_seq, Ordering::AcqRel)
            .max(tail.last_seq);
        let n = batch.len() as u64;
        // Ship the primary's commit versionstamps ahead of the frames. A
        // commit's stamp is recorded just after its WAL frame is written, so
        // wait briefly for in-flight stamped commits to settle.
        {
            let eng = Arc::clone(&inner.engine);
            let (lo, hi) = (
                batch.first().map(|f| f.seq).unwrap_or(0),
                batch.last().map(|f| f.seq).unwrap_or(0),
            );
            let stamps = tokio::task::spawn_blocking(move || {
                let t0 = Instant::now();
                while eng.commit_stamps_pending() > 0
                    && eng.commit_stamps_in(hi, hi).is_empty()
                    && t0.elapsed() < Duration::from_millis(250)
                {
                    std::thread::sleep(Duration::from_micros(100));
                }
                eng.commit_stamps_in(lo, hi)
            })
            .await
            .map_err(|e| ReplicationError::Wal(e.to_string()))?;
            if !stamps.is_empty() {
                send(inner, wr, &Message::Stamps(stamps)).await?;
            }
        }
        send(
            inner,
            wr,
            &Message::Frames {
                epoch: inner.fence_epoch,
                tip,
                frames: batch,
            },
        )
        .await?;
        inner.metrics.frames_sent.fetch_add(n, Ordering::Relaxed);
        last_send = Instant::now();
    }
}
