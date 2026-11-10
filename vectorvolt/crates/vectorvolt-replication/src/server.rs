//! [`ReplicationServer`] — the leader side of WAL-shipping replication.
//!
//! For every follower connection the server reads the follower's last applied
//! LSN, decides between streaming and a snapshot, ships committed WAL frames
//! (CRC-framed, still encrypted when the table is), and records the follower's
//! acks so writers can wait for them according to the configured mode.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::{Condvar, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use vectorvolt_engine::TableEngine;

use crate::config::{ReplicationConfig, ReplicationMode, ReplicationRole, RoleCell};
use crate::error::{ReplResult, ReplicationError};
use crate::metrics::ReplicationMetrics;
use crate::proto::{read_message, write_message, Message, PROTO_VERSION};
use crate::wal_tail::{wal_bounds, WalTail};

/// On snapshot, streaming restarts this many frames before the WAL tip that
/// was read before the row scan. A row write is appended to the WAL slightly
/// before it becomes visible to a scan, so frames right at the tip may be
/// missing from the scan; replaying them is idempotent (final-state ops).
const SNAPSHOT_OVERLAP: u64 = 4096;

/// Per-follower acknowledgement state.
struct Slot {
    addr: SocketAddr,
    applied: AtomicU64,
    durable: AtomicU64,
    /// ms (server epoch) of the last ack that reached the leader tip.
    caught_up_ms: AtomicU64,
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
    shutdown_notify: Notify,
    wal_notify: Notify,
    ack_notify: Notify,
    ack_lock: Mutex<()>,
    ack_cv: Condvar,
    epoch: Instant,
}

impl Inner {
    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    fn acks_for(&self, lsn: u64, durable: bool) -> usize {
        self.followers
            .lock()
            .values()
            .filter(|s| {
                let v = if durable {
                    s.durable.load(Ordering::Acquire)
                } else {
                    s.applied.load(Ordering::Acquire)
                };
                v >= lsn
            })
            .count()
    }

    fn check_role(&self) -> ReplResult<()> {
        match self.role.get() {
            ReplicationRole::Follower => Err(ReplicationError::NotLeader),
            _ => Ok(()),
        }
    }

    fn timeout_err(&self, lsn: u64, needed: usize, durable: bool) -> ReplicationError {
        self.metrics.ack_timeouts.fetch_add(1, Ordering::Relaxed);
        ReplicationError::AckTimeout {
            lsn,
            needed,
            got: self.acks_for(lsn, durable),
            timeout: self.cfg.ack_timeout,
        }
    }
}

/// Leader-side replication endpoint. Cheap to clone via `Arc`.
pub struct ReplicationServer {
    inner: Arc<Inner>,
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
        cfg.validate()?;
        let wal_dir = wal_dir.into();
        let tip = wal_bounds(&wal_dir)?.tip;
        let listener = TcpListener::bind(listen).await?;
        let local_addr = listener.local_addr()?;
        let inner = Arc::new(Inner {
            engine,
            wal_dir,
            cfg,
            role,
            metrics: Arc::new(ReplicationMetrics::default()),
            followers: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            tip: AtomicU64::new(tip),
            shutdown: AtomicBool::new(false),
            shutdown_notify: Notify::new(),
            wal_notify: Notify::new(),
            ack_notify: Notify::new(),
            ack_lock: Mutex::new(()),
            ack_cv: Condvar::new(),
            epoch: Instant::now(),
        });
        let accept_inner = Arc::clone(&inner);
        let handle = tokio::spawn(async move {
            loop {
                let accepted = tokio::select! {
                    r = listener.accept() => r,
                    _ = accept_inner.shutdown_notify.notified() => break,
                };
                if accept_inner.shutdown.load(Ordering::Acquire) {
                    break;
                }
                match accepted {
                    Ok((stream, peer)) => {
                        let conn_inner = Arc::clone(&accept_inner);
                        tokio::spawn(async move {
                            if let Err(e) = handle_conn(Arc::clone(&conn_inner), stream, peer).await
                            {
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
        });
        Ok(Arc::new(Self {
            inner,
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
        let tip = self.inner.tip.load(Ordering::Acquire);
        let now = self.inner.now_ms();
        let mut v: Vec<FollowerInfo> = self
            .inner
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

    /// Number of followers currently connected.
    pub fn follower_count(&self) -> usize {
        self.inner.followers.lock().len()
    }

    /// Wake streaming connections now instead of at the next poll. Call after
    /// a local engine write to cut replication latency.
    pub fn notify_write(&self) {
        self.inner.wal_notify.notify_waiters();
    }

    /// Wait until the write that produced WAL sequence `lsn` is replicated as
    /// the configured mode requires:
    ///
    /// - `Async`: returns immediately.
    /// - `SemiSync`: one follower has applied it.
    /// - `Sync`: `replication_factor - 1` followers have durably applied it.
    ///
    /// Returns [`ReplicationError::AckTimeout`] if the acks do not arrive
    /// within `ack_timeout` — never a silent success.
    pub async fn wait_for_replication(&self, lsn: u64) -> ReplResult<()> {
        let inner = &self.inner;
        let needed = inner.cfg.required_acks();
        if needed == 0 {
            return Ok(());
        }
        inner.check_role()?;
        let durable = inner.cfg.mode == ReplicationMode::Sync;
        let deadline = Instant::now() + inner.cfg.ack_timeout;
        self.notify_write();
        loop {
            if inner.acks_for(lsn, durable) >= needed {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(inner.timeout_err(lsn, needed, durable));
            }
            let wait = (deadline - now).min(Duration::from_millis(25));
            tokio::select! {
                _ = inner.ack_notify.notified() => {}
                _ = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// Blocking variant of [`wait_for_replication`](Self::wait_for_replication)
    /// for synchronous callers (must not be called from an async task).
    pub fn wait_for_replication_blocking(&self, lsn: u64) -> ReplResult<()> {
        let inner = &self.inner;
        let needed = inner.cfg.required_acks();
        if needed == 0 {
            return Ok(());
        }
        inner.check_role()?;
        let durable = inner.cfg.mode == ReplicationMode::Sync;
        let deadline = Instant::now() + inner.cfg.ack_timeout;
        self.notify_write();
        let mut guard = inner.ack_lock.lock();
        loop {
            if inner.acks_for(lsn, durable) >= needed {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(inner.timeout_err(lsn, needed, durable));
            }
            let wait = (deadline - now).min(Duration::from_millis(25));
            inner.ack_cv.wait_for(&mut guard, wait);
        }
    }

    /// Insert on the leader engine, then wait per the replication mode.
    /// Returns the WAL sequence of the write.
    pub async fn insert_replicated(&self, pk: u64, values: Vec<u64>) -> ReplResult<u64> {
        self.inner.check_role()?;
        let tip = self.inner.engine.insert(pk, values)?;
        self.wait_for_replication(tip.sequence).await?;
        Ok(tip.sequence)
    }

    /// Delete on the leader engine, then wait per the replication mode.
    pub async fn delete_replicated(&self, pk: u64) -> ReplResult<u64> {
        self.inner.check_role()?;
        let tip = self.inner.engine.delete(pk)?;
        self.wait_for_replication(tip.sequence).await?;
        Ok(tip.sequence)
    }

    /// Stop accepting and drop every follower stream.
    pub fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        self.inner.shutdown_notify.notify_waiters();
        self.inner.wal_notify.notify_waiters();
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

// ── per-connection ────────────────────────────────────────────────────────────

struct SlotGuard {
    inner: Arc<Inner>,
    id: u64,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.inner.followers.lock().remove(&self.id);
        self.inner.metrics.connected.store(
            (!self.inner.followers.lock().is_empty()) as u64,
            Ordering::Relaxed,
        );
    }
}

async fn send(
    inner: &Inner,
    w: &mut tokio::net::tcp::OwnedWriteHalf,
    msg: &Message,
) -> ReplResult<()> {
    let n = write_message(w, msg).await?;
    inner
        .metrics
        .bytes_sent
        .fetch_add(n as u64, Ordering::Relaxed);
    Ok(())
}

async fn handle_conn(inner: Arc<Inner>, stream: TcpStream, peer: SocketAddr) -> ReplResult<()> {
    stream.set_nodelay(true)?;
    let (mut rd, mut wr) = stream.into_split();

    let (hello, _) = tokio::time::timeout(Duration::from_secs(5), read_message(&mut rd))
        .await
        .map_err(|_| ReplicationError::Protocol("hello timeout".into()))??;
    let applied = match hello {
        Message::Hello {
            version,
            applied_lsn,
        } if version == PROTO_VERSION => applied_lsn,
        Message::Hello { version, .. } => {
            let _ = send(
                &inner,
                &mut wr,
                &Message::Error(format!("unsupported protocol version {version}")),
            )
            .await;
            return Err(ReplicationError::Protocol("bad version".into()));
        }
        _ => return Err(ReplicationError::Protocol("expected hello".into())),
    };
    if inner.role.get() == ReplicationRole::Follower {
        let _ = send(&inner, &mut wr, &Message::Error("not the leader".into())).await;
        return Err(ReplicationError::NotLeader);
    }

    // Decide: stream from the follower's LSN, or snapshot first.
    let wal_dir = inner.wal_dir.clone();
    let bounds = tokio::task::spawn_blocking(move || wal_bounds(&wal_dir))
        .await
        .map_err(|e| ReplicationError::Wal(e.to_string()))??;
    inner.tip.fetch_max(bounds.tip, Ordering::AcqRel);
    let floor = bounds.first_seq.map(|f| f.saturating_sub(1)).unwrap_or(0);
    let needs_snapshot = applied > bounds.tip || applied < floor;
    let start_lsn = if needs_snapshot {
        bounds.tip.saturating_sub(SNAPSHOT_OVERLAP).max(floor)
    } else {
        applied
    };

    let id = inner.next_id.fetch_add(1, Ordering::Relaxed);
    let slot = Arc::new(Slot {
        addr: peer,
        applied: AtomicU64::new(if needs_snapshot { 0 } else { applied }),
        durable: AtomicU64::new(if needs_snapshot { 0 } else { applied }),
        caught_up_ms: AtomicU64::new(inner.now_ms()),
    });
    inner.followers.lock().insert(id, Arc::clone(&slot));
    inner.metrics.connected.store(1, Ordering::Relaxed);
    let _guard = SlotGuard {
        inner: Arc::clone(&inner),
        id,
    };

    send(
        &inner,
        &mut wr,
        &Message::Welcome {
            first_lsn: bounds.first_seq.unwrap_or(1),
            tip: bounds.tip,
            snapshot: needs_snapshot,
            start_lsn,
        },
    )
    .await?;

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
                        applied_lsn,
                        durable_lsn,
                    },
                    n,
                )) => {
                    ack_inner
                        .metrics
                        .bytes_received
                        .fetch_add(n as u64, Ordering::Relaxed);
                    ack_slot.applied.fetch_max(applied_lsn, Ordering::AcqRel);
                    ack_slot.durable.fetch_max(durable_lsn, Ordering::AcqRel);
                    if applied_lsn >= ack_inner.tip.load(Ordering::Acquire) {
                        ack_slot
                            .caught_up_ms
                            .store(ack_inner.now_ms(), Ordering::Release);
                    }
                    {
                        let _g = ack_inner.ack_lock.lock();
                        ack_inner.ack_cv.notify_all();
                    }
                    ack_inner.ack_notify.notify_waiters();
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

    let result = stream_loop(&inner, &mut wr, &dead, needs_snapshot, start_lsn, &slot).await;
    ack_task.abort();
    result
}

async fn stream_loop(
    inner: &Arc<Inner>,
    wr: &mut tokio::net::tcp::OwnedWriteHalf,
    dead: &AtomicBool,
    needs_snapshot: bool,
    start_lsn: u64,
    _slot: &Slot,
) -> ReplResult<()> {
    if needs_snapshot {
        let engine = Arc::clone(&inner.engine);
        let rows = tokio::task::spawn_blocking(move || -> ReplResult<Vec<(u64, Vec<u64>)>> {
            // `scan` can return stale duplicates of an updated row; `get` is
            // the authority, and drops rows that were deleted.
            let mut seen = HashSet::new();
            let mut rows = Vec::new();
            for r in engine.scan(0, 0, u64::MAX)? {
                if !seen.insert(r.pk) {
                    continue;
                }
                if let Some(cur) = engine.get(r.pk)? {
                    rows.push((cur.pk, cur.values));
                }
            }
            Ok(rows)
        })
        .await
        .map_err(|e| ReplicationError::Engine(e.to_string()))??;
        for chunk in rows.chunks(inner.cfg.snapshot_batch_rows) {
            send(inner, wr, &Message::SnapshotRows(chunk.to_vec())).await?;
        }
        send(
            inner,
            wr,
            &Message::SnapshotEnd {
                snapshot_lsn: start_lsn,
            },
        )
        .await?;
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
                send(inner, wr, &Message::Heartbeat { tip }).await?;
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
        send(inner, wr, &Message::Frames { tip, frames: batch }).await?;
        inner.metrics.frames_sent.fetch_add(n, Ordering::Relaxed);
        last_send = Instant::now();
    }
}
