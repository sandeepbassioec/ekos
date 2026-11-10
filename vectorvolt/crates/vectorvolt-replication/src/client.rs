//! [`ReplicationClient`] — the follower side of WAL-shipping replication.
//!
//! Connects to the leader with its last applied LSN, applies the frames (or a
//! snapshot followed by frames) it receives to the local engine, persists its
//! position, and acks. Reconnects with exponential backoff.
//!
//! Frames are decoded exactly as the engine's own WAL replay does:
//! `vectorvolt_txn::manager::open_wal_payload` (decrypting with the table key
//! when the WAL is encrypted at rest) and then `Mutation::decode`. The decoded
//! mutations are applied as final-state operations through
//! [`TableEngine::apply_batch`], which is idempotent, so a frame re-delivered
//! after a crash between "apply" and "persist position" is harmless.

use std::collections::HashSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use vectorvolt_engine::TableEngine;
use vectorvolt_txn::Mutation;
use vectorvolt_wal::segment::SegmentHeader;

use crate::config::{ReplicationConfig, ReplicationRole, RoleCell};
use crate::error::{ReplResult, ReplicationError};
use crate::metrics::ReplicationMetrics;
use crate::proto::{read_message, write_message, Message, WireFrame, PROTO_VERSION};

/// File (inside the follower's data dir) holding the last applied leader LSN.
pub const POSITION_FILE: &str = "replication.pos";

const POS_MAGIC: &[u8; 4] = b"VRPS";

/// Load the persisted leader LSN; 0 if absent or unreadable.
pub fn load_position(dir: &Path) -> u64 {
    let Ok(b) = std::fs::read(dir.join(POSITION_FILE)) else {
        return 0;
    };
    if b.len() != 16 || &b[0..4] != POS_MAGIC {
        return 0;
    }
    let lsn = u64::from_le_bytes(b[4..12].try_into().unwrap());
    let crc = u32::from_le_bytes(b[12..16].try_into().unwrap());
    if crc32fast::hash(&b[0..12]) != crc {
        tracing::warn!("replication position file is corrupt; restarting from LSN 0");
        return 0;
    }
    lsn
}

/// Durably persist `lsn` (write temp, fsync, rename).
pub fn save_position(dir: &Path, lsn: u64) -> std::io::Result<()> {
    let mut b = Vec::with_capacity(16);
    b.extend_from_slice(POS_MAGIC);
    b.extend_from_slice(&lsn.to_le_bytes());
    let crc = crc32fast::hash(&b);
    b.extend_from_slice(&crc.to_le_bytes());
    let tmp = dir.join(format!("{POSITION_FILE}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&b)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, dir.join(POSITION_FILE))
}

struct Inner {
    engine: Arc<TableEngine>,
    data_dir: PathBuf,
    key: Option<Arc<vectorvolt_crypto::KeyStore>>,
    cfg: ReplicationConfig,
    role: RoleCell,
    leader_addr: String,
    metrics: Arc<ReplicationMetrics>,
    applied: AtomicU64,
    shutdown: AtomicBool,
    shutdown_notify: Notify,
    last_error: Mutex<Option<String>>,
}

/// Follower-side replication driver. Spawns its connect/apply loop on the
/// current tokio runtime.
pub struct ReplicationClient {
    inner: Arc<Inner>,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl ReplicationClient {
    /// Start replicating from `leader_addr` (`host:port`).
    ///
    /// `data_dir` is the follower table's directory (where the position file
    /// lives). `key` is the table's at-rest key if its WAL is encrypted — the
    /// same key the engine and `WalSinkBridge::with_key` use.
    pub fn start(
        engine: Arc<TableEngine>,
        data_dir: impl Into<PathBuf>,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: impl Into<String>,
    ) -> ReplResult<Arc<Self>> {
        cfg.validate()?;
        let data_dir = data_dir.into();
        let applied = load_position(&data_dir);
        let metrics = Arc::new(ReplicationMetrics::default());
        metrics.applied_lsn.store(applied, Ordering::Release);
        metrics.leader_tip_lsn.store(applied, Ordering::Release);
        let inner = Arc::new(Inner {
            engine,
            data_dir,
            key,
            cfg,
            role,
            leader_addr: leader_addr.into(),
            metrics,
            applied: AtomicU64::new(applied),
            shutdown: AtomicBool::new(false),
            shutdown_notify: Notify::new(),
            last_error: Mutex::new(None),
        });
        let task_inner = Arc::clone(&inner);
        let handle = tokio::spawn(async move { run_loop(task_inner).await });
        Ok(Arc::new(Self {
            inner,
            task: Mutex::new(Some(handle)),
        }))
    }

    /// Highest leader LSN applied (and persisted).
    pub fn applied_lsn(&self) -> u64 {
        self.inner.applied.load(Ordering::Acquire)
    }

    pub fn metrics(&self) -> Arc<ReplicationMetrics> {
        Arc::clone(&self.inner.metrics)
    }

    pub fn role(&self) -> &RoleCell {
        &self.inner.role
    }

    /// Whether a stream to the leader is currently established.
    pub fn is_connected(&self) -> bool {
        self.inner.metrics.connected.load(Ordering::Relaxed) != 0
    }

    /// The most recent session error, if any (diagnostics).
    pub fn last_error(&self) -> Option<String> {
        self.inner.last_error.lock().clone()
    }

    /// Wait until `applied_lsn() >= lsn`; `false` on timeout.
    pub async fn wait_for_lsn(&self, lsn: u64, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if self.applied_lsn() >= lsn {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    /// Stop replicating. The local engine and position file are left intact.
    pub fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        self.inner.shutdown_notify.notify_waiters();
        if let Some(h) = self.task.lock().take() {
            h.abort();
        }
        self.inner.metrics.connected.store(0, Ordering::Relaxed);
    }
}

impl Drop for ReplicationClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn run_loop(inner: Arc<Inner>) {
    let mut backoff = inner.cfg.backoff_initial;
    loop {
        if inner.shutdown.load(Ordering::Acquire) {
            return;
        }
        // A promoted node stops pulling; it resumes if demoted again.
        if inner.role.get() != ReplicationRole::Follower {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                _ = inner.shutdown_notify.notified() => {}
            }
            continue;
        }
        let mut progressed = false;
        match session(&inner, &mut progressed).await {
            Ok(()) => {}
            Err(e) => {
                if matches!(e, ReplicationError::Corrupt(_)) {
                    inner
                        .metrics
                        .rejected_frames
                        .fetch_add(1, Ordering::Relaxed);
                }
                tracing::debug!("replication session ended: {e}");
                *inner.last_error.lock() = Some(e.to_string());
            }
        }
        inner.metrics.connected.store(0, Ordering::Relaxed);
        inner.metrics.reconnects.fetch_add(1, Ordering::Relaxed);
        if progressed {
            backoff = inner.cfg.backoff_initial;
        }
        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = inner.shutdown_notify.notified() => {}
        }
        backoff = (backoff * 2).min(inner.cfg.backoff_max);
    }
}

async fn recv(inner: &Inner, rd: &mut tokio::net::tcp::OwnedReadHalf) -> ReplResult<Message> {
    let (msg, n) = tokio::time::timeout(inner.cfg.read_timeout, read_message(rd))
        .await
        .map_err(|_| ReplicationError::Protocol("leader read timeout".into()))??;
    inner
        .metrics
        .bytes_received
        .fetch_add(n as u64, Ordering::Relaxed);
    Ok(msg)
}

async fn session(inner: &Arc<Inner>, progressed: &mut bool) -> ReplResult<()> {
    let stream = tokio::time::timeout(
        Duration::from_secs(3),
        TcpStream::connect(inner.leader_addr.as_str()),
    )
    .await
    .map_err(|_| ReplicationError::Protocol("connect timeout".into()))??;
    stream.set_nodelay(true)?;
    let (mut rd, mut wr) = stream.into_split();

    let applied = inner.applied.load(Ordering::Acquire);
    write_message(
        &mut wr,
        &Message::Hello {
            version: PROTO_VERSION,
            applied_lsn: applied,
        },
    )
    .await?;
    let (snapshot, start_lsn, tip) = match recv(inner, &mut rd).await? {
        Message::Welcome {
            snapshot,
            start_lsn,
            tip,
            ..
        } => (snapshot, start_lsn, tip),
        Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
        _ => return Err(ReplicationError::Protocol("expected welcome".into())),
    };
    inner.metrics.connected.store(1, Ordering::Relaxed);
    inner.metrics.observe_tip(tip);
    *progressed = true;

    if snapshot {
        receive_snapshot(inner, &mut rd, start_lsn).await?;
    }

    loop {
        if inner.shutdown.load(Ordering::Acquire) {
            return Ok(());
        }
        if inner.role.get() != ReplicationRole::Follower {
            return Ok(());
        }
        let msg = recv(inner, &mut rd).await?;
        // Promoted while waiting: do not apply leader frames to a writable node.
        if inner.role.get() != ReplicationRole::Follower {
            return Ok(());
        }
        match msg {
            Message::Frames { tip, frames } => {
                inner.metrics.observe_tip(tip);
                let before = inner.applied.load(Ordering::Acquire);
                let n = frames.len() as u64;
                let engine = Arc::clone(&inner.engine);
                let key = inner.key.clone();
                let new_applied = tokio::task::spawn_blocking(move || {
                    apply_frames(&engine, key.as_deref(), frames, before)
                })
                .await
                .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                if new_applied == before {
                    continue;
                }
                // Tell the leader the frames are applied, then make the
                // position durable before publishing it locally and acking
                // durability.
                write_message(
                    &mut wr,
                    &Message::Ack {
                        applied_lsn: new_applied,
                        durable_lsn: before,
                    },
                )
                .await?;
                let dir = inner.data_dir.clone();
                tokio::task::spawn_blocking(move || save_position(&dir, new_applied))
                    .await
                    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                inner.metrics.frames_applied.fetch_add(n, Ordering::Relaxed);
                inner.applied.store(new_applied, Ordering::Release);
                inner
                    .metrics
                    .applied_lsn
                    .store(new_applied, Ordering::Release);
                inner.metrics.update_lag_state();
                write_message(
                    &mut wr,
                    &Message::Ack {
                        applied_lsn: new_applied,
                        durable_lsn: new_applied,
                    },
                )
                .await?;
            }
            Message::Heartbeat { tip } => {
                inner.metrics.observe_tip(tip);
                let a = inner.applied.load(Ordering::Acquire);
                write_message(
                    &mut wr,
                    &Message::Ack {
                        applied_lsn: a,
                        durable_lsn: a,
                    },
                )
                .await?;
            }
            Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
            _ => return Err(ReplicationError::Protocol("unexpected message".into())),
        }
    }
}

async fn receive_snapshot(
    inner: &Arc<Inner>,
    rd: &mut tokio::net::tcp::OwnedReadHalf,
    start_lsn: u64,
) -> ReplResult<()> {
    // Position 0 *before* wiping: if the session dies half-way, the next one
    // must not resume streaming over a half-built replica.
    let dir = inner.data_dir.clone();
    let engine = Arc::clone(&inner.engine);
    tokio::task::spawn_blocking(move || -> ReplResult<()> {
        save_position(&dir, 0)?;
        wipe(&engine)
    })
    .await
    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
    inner.applied.store(0, Ordering::Release);
    inner.metrics.applied_lsn.store(0, Ordering::Release);

    loop {
        match recv(inner, rd).await? {
            Message::SnapshotRows(rows) => {
                let engine = Arc::clone(&inner.engine);
                tokio::task::spawn_blocking(move || -> ReplResult<()> {
                    let ops = rows.into_iter().map(|(pk, v)| (pk, Some(v))).collect();
                    engine.apply_batch(ops)?;
                    Ok(())
                })
                .await
                .map_err(|e| ReplicationError::Engine(e.to_string()))??;
            }
            Message::SnapshotEnd { snapshot_lsn } => {
                debug_assert_eq!(snapshot_lsn, start_lsn);
                let dir = inner.data_dir.clone();
                tokio::task::spawn_blocking(move || save_position(&dir, snapshot_lsn))
                    .await
                    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                inner.applied.store(snapshot_lsn, Ordering::Release);
                inner
                    .metrics
                    .applied_lsn
                    .store(snapshot_lsn, Ordering::Release);
                inner.metrics.update_lag_state();
                inner
                    .metrics
                    .snapshots_applied
                    .fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
            _ => return Err(ReplicationError::Protocol("unexpected in snapshot".into())),
        }
    }
}

/// Delete every live row so a snapshot can be installed over stale state.
fn wipe(engine: &TableEngine) -> ReplResult<()> {
    let mut seen = HashSet::new();
    let mut ops = Vec::new();
    for r in engine.scan(0, 0, u64::MAX)? {
        if seen.insert(r.pk) {
            ops.push((r.pk, None));
        }
    }
    for chunk in ops.chunks(4096) {
        engine.apply_batch(chunk.to_vec())?;
    }
    Ok(())
}

/// Decode `frames` the way the engine's WAL replay does and apply them.
/// Returns the new applied LSN.
fn apply_frames(
    engine: &TableEngine,
    key: Option<&vectorvolt_crypto::KeyStore>,
    frames: Vec<WireFrame>,
    applied: u64,
) -> ReplResult<u64> {
    let mut expected = applied + 1;
    let mut ops: Vec<(u64, Option<Vec<u64>>)> = Vec::new();
    let mut last = applied;
    for f in frames {
        if f.seq <= applied {
            continue; // duplicate of something already applied
        }
        if f.seq != expected {
            return Err(ReplicationError::Protocol(format!(
                "gap in replication stream: expected lsn {expected}, got {}",
                f.seq
            )));
        }
        expected += 1;
        last = f.seq;
        let header = SegmentHeader::with_key_id(0, 0, f.key_id);
        let payload = vectorvolt_txn::manager::open_wal_payload(key, &header, f.seq, f.payload)
            .map_err(|e| ReplicationError::Decrypt(format!("lsn {}: {}", e.seq, e.reason)))?;
        match Mutation::decode(&payload) {
            Ok(Mutation::Insert { pk, values })
            | Ok(Mutation::Upsert { pk, values })
            | Ok(Mutation::Update { pk, values }) => ops.push((pk, Some(values))),
            Ok(Mutation::Delete { pk }) => ops.push((pk, None)),
            Ok(Mutation::InsertBatch { rows }) | Ok(Mutation::UpsertBatch { rows }) => {
                ops.extend(rows.into_iter().map(|(pk, v)| (pk, Some(v))));
            }
            Ok(Mutation::DeleteBatch { pks }) => ops.extend(pks.into_iter().map(|pk| (pk, None))),
            Ok(Mutation::TxnBatch { ops: o }) => ops.extend(o),
            // Checkpoints, DDL and non-mutation payloads carry no row data.
            Ok(_) | Err(_) => {}
        }
    }
    if !ops.is_empty() {
        // Keep only the last op per pk (ops are final states, order matters
        // per key only), preserving relative order.
        let mut seen = HashSet::new();
        let mut dedup: Vec<_> = ops
            .into_iter()
            .rev()
            .filter(|(pk, _)| seen.insert(*pk))
            .collect();
        dedup.reverse();
        engine.apply_batch(dedup)?;
    }
    Ok(last)
}
