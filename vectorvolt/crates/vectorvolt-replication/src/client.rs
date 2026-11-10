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

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use vectorvolt_engine::TableEngine;
use vectorvolt_txn::Mutation;
use vectorvolt_wal::segment::SegmentHeader;

use crate::config::{EpochCell, ReplicationConfig, ReplicationRole, RoleCell};
use crate::error::{ReplResult, ReplicationError};
use crate::lineage::LineageStore;
use crate::metrics::ReplicationMetrics;
use crate::proto::{read_message, write_message, Message, WireFrame, PROTO_VERSION};
use crate::tls::{Conn, ConnRead, ConnWrite, ReplicationTlsClient};
use crate::wal_tail::{wal_bounds, WalTail};

/// File (inside the follower's data dir) holding the last applied leader LSN.
pub const POSITION_FILE: &str = "replication.pos";

const POS_MAGIC: &[u8; 4] = b"VRPS";
const POS_MAGIC_V2: &[u8; 4] = b"VRP2";

/// Where a follower stands: `lsn` in the LSN space of the primary of `epoch`.
/// `primary` marks a node that was itself the primary of `epoch` (its `lsn`
/// is then its own WAL tip, which keeps moving; read it from the engine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub epoch: u64,
    pub lsn: u64,
    pub primary: bool,
}

/// Load the persisted position; the default (epoch 0, LSN 0) if absent or unreadable.
pub fn load_position_full(dir: &Path) -> Position {
    let Ok(b) = std::fs::read(dir.join(POSITION_FILE)) else {
        return Position::default();
    };
    let corrupt = || {
        tracing::warn!("replication position file is corrupt; restarting from LSN 0");
        Position::default()
    };
    match (b.len(), b.get(0..4)) {
        (16, Some(m)) if m == POS_MAGIC => {
            let lsn = u64::from_le_bytes(b[4..12].try_into().unwrap());
            let crc = u32::from_le_bytes(b[12..16].try_into().unwrap());
            if crc32fast::hash(&b[0..12]) != crc {
                return corrupt();
            }
            Position {
                epoch: 0,
                lsn,
                primary: false,
            }
        }
        (25, Some(m)) if m == POS_MAGIC_V2 => {
            let crc = u32::from_le_bytes(b[21..25].try_into().unwrap());
            if crc32fast::hash(&b[0..21]) != crc {
                return corrupt();
            }
            Position {
                epoch: u64::from_le_bytes(b[4..12].try_into().unwrap()),
                lsn: u64::from_le_bytes(b[12..20].try_into().unwrap()),
                primary: b[20] != 0,
            }
        }
        _ => corrupt(),
    }
}

/// Load the persisted leader LSN; 0 if absent or unreadable.
pub fn load_position(dir: &Path) -> u64 {
    load_position_full(dir).lsn
}

/// Durably persist `pos` (write temp, fsync, rename).
pub fn save_position_full(dir: &Path, pos: &Position) -> std::io::Result<()> {
    let mut b = Vec::with_capacity(25);
    b.extend_from_slice(POS_MAGIC_V2);
    b.extend_from_slice(&pos.epoch.to_le_bytes());
    b.extend_from_slice(&pos.lsn.to_le_bytes());
    b.push(pos.primary as u8);
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

/// Durably persist `lsn` (epoch 0).
pub fn save_position(dir: &Path, lsn: u64) -> std::io::Result<()> {
    save_position_full(
        dir,
        &Position {
            epoch: 0,
            lsn,
            primary: false,
        },
    )
}

struct Inner {
    engine: Arc<TableEngine>,
    data_dir: PathBuf,
    key: Option<Arc<vectorvolt_crypto::KeyStore>>,
    cfg: ReplicationConfig,
    role: RoleCell,
    leader_addr: String,
    tls: Option<ReplicationTlsClient>,
    /// `Some(name)` when this is one table of a multi-table replication.
    table: Option<String>,
    metrics: Arc<ReplicationMetrics>,
    applied: AtomicU64,
    /// Epoch of the primary whose LSN space `applied` is in.
    pos_epoch: AtomicU64,
    /// Highest primary epoch this node has seen (failover mode only).
    known: Option<EpochCell>,
    /// Translation table to the primaries followed (failover mode only).
    lineage: Option<Arc<LineageStore>>,
    shutdown: AtomicBool,
    shutdown_notify: Notify,
    last_error: Mutex<Option<String>>,
}

impl Inner {
    fn known_epoch(&self) -> u64 {
        self.known.as_ref().map_or(0, EpochCell::get)
    }

    fn history(&self, pos_epoch: u64, applied: u64) -> Vec<(u64, u64)> {
        match &self.lineage {
            Some(l) => l
                .lock()
                .history((pos_epoch, applied), self.engine.wal_tip_sequence()),
            None => Vec::new(),
        }
    }

    /// Record a new position: `lsn` in `epoch`'s space, durably, in the lineage
    /// and in memory. `reset` replaces the whole lineage (a snapshot was installed).
    async fn set_position(&self, epoch: u64, lsn: u64, reset: bool) -> ReplResult<()> {
        let dir = self.data_dir.clone();
        let engine = Arc::clone(&self.engine);
        let lineage = self.lineage.clone();
        tokio::task::spawn_blocking(move || -> ReplResult<()> {
            let local = engine.wal_tip_sequence();
            if let Some(l) = lineage {
                let mut l = l.lock();
                if reset {
                    l.reset_to(epoch, lsn, local)?;
                } else {
                    l.record_applied(epoch, lsn, local)?;
                }
            }
            save_position_full(
                &dir,
                &Position {
                    epoch,
                    lsn,
                    primary: false,
                },
            )?;
            Ok(())
        })
        .await
        .map_err(|e| ReplicationError::Engine(e.to_string()))??;
        if self.pos_epoch.swap(epoch, Ordering::AcqRel) != epoch {
            self.metrics.leader_tip_lsn.store(lsn, Ordering::Release);
        }
        self.applied.store(lsn, Ordering::Release);
        self.metrics.applied_lsn.store(lsn, Ordering::Release);
        self.metrics.update_lag_state();
        Ok(())
    }
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
        Self::start_tls(engine, data_dir, key, cfg, role, leader_addr, None)
    }

    /// [`start`](Self::start) over TLS / mTLS when `tls` is set.
    pub fn start_tls(
        engine: Arc<TableEngine>,
        data_dir: impl Into<PathBuf>,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: impl Into<String>,
        tls: Option<ReplicationTlsClient>,
    ) -> ReplResult<Arc<Self>> {
        Self::spawn(
            engine,
            data_dir.into(),
            key,
            cfg,
            role,
            leader_addr.into(),
            tls,
            None,
        )
    }

    /// Failover-aware client: tracks the highest primary epoch seen in `known`
    /// (anything from a lower epoch is dropped) and keeps `lineage` so that, after
    /// the primary changes, it resumes from a common point with the new primary
    /// instead of re-syncing from a snapshot.
    #[allow(clippy::too_many_arguments)]
    pub fn start_epoch(
        engine: Arc<TableEngine>,
        data_dir: impl Into<PathBuf>,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: impl Into<String>,
        known: EpochCell,
        lineage: Arc<LineageStore>,
    ) -> ReplResult<Arc<Self>> {
        Self::spawn_with(
            engine,
            data_dir.into(),
            key,
            cfg,
            role,
            leader_addr.into(),
            None,
            None,
            Some(known),
            Some(lineage),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn spawn(
        engine: Arc<TableEngine>,
        data_dir: PathBuf,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: String,
        tls: Option<ReplicationTlsClient>,
        table: Option<String>,
    ) -> ReplResult<Arc<Self>> {
        Self::spawn_with(
            engine,
            data_dir,
            key,
            cfg,
            role,
            leader_addr,
            tls,
            table,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_with(
        engine: Arc<TableEngine>,
        data_dir: PathBuf,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: String,
        tls: Option<ReplicationTlsClient>,
        table: Option<String>,
        known: Option<EpochCell>,
        lineage: Option<Arc<LineageStore>>,
    ) -> ReplResult<Arc<Self>> {
        cfg.validate()?;
        let pos = load_position_full(&data_dir);
        let applied = pos.lsn;
        let metrics = Arc::new(ReplicationMetrics::default());
        metrics.applied_lsn.store(applied, Ordering::Release);
        metrics.leader_tip_lsn.store(applied, Ordering::Release);
        let inner = Arc::new(Inner {
            engine,
            data_dir,
            key,
            cfg,
            role,
            leader_addr,
            tls,
            table,
            metrics,
            applied: AtomicU64::new(applied),
            pos_epoch: AtomicU64::new(pos.epoch),
            known,
            lineage,
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

/// Open a connection to the leader: TCP, then TLS when configured.
pub(crate) async fn connect(addr: &str, tls: Option<&ReplicationTlsClient>) -> ReplResult<Conn> {
    let stream = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr))
        .await
        .map_err(|_| ReplicationError::Protocol("connect timeout".into()))??;
    stream.set_nodelay(true)?;
    Ok(match tls {
        Some(t) => {
            let name = t.name_for(addr)?;
            let s = tokio::time::timeout(Duration::from_secs(5), t.connector.connect(name, stream))
                .await
                .map_err(|_| ReplicationError::Protocol("tls handshake timeout".into()))??;
            Box::new(s)
        }
        None => Box::new(stream),
    })
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

async fn recv(inner: &Inner, rd: &mut ConnRead) -> ReplResult<Message> {
    let (msg, n) = tokio::time::timeout(inner.cfg.read_timeout, read_message(rd))
        .await
        .map_err(|_| ReplicationError::Protocol("leader read timeout".into()))??;
    inner
        .metrics
        .bytes_received
        .fetch_add(n as u64, Ordering::Relaxed);
    Ok(msg)
}

/// Drop-and-end rule for epochs: anything below what this node already knows,
/// or not from the session's own primary, is not applied.
fn check_epoch(inner: &Inner, session_epoch: u64, msg_epoch: u64) -> ReplResult<()> {
    if msg_epoch < inner.known_epoch().max(session_epoch) {
        inner
            .metrics
            .stale_epoch_frames
            .fetch_add(1, Ordering::Relaxed);
        return Err(ReplicationError::Protocol(format!(
            "dropped message from stale epoch {msg_epoch} (known {})",
            inner.known_epoch().max(session_epoch)
        )));
    }
    if msg_epoch != session_epoch {
        return Err(ReplicationError::Protocol(format!(
            "epoch changed mid-stream ({session_epoch} -> {msg_epoch}); reconnecting"
        )));
    }
    Ok(())
}

async fn send_ack(wr: &mut ConnWrite, epoch: u64, applied: u64, durable: u64) -> ReplResult<()> {
    write_message(
        wr,
        &Message::Ack {
            epoch,
            applied_lsn: applied,
            durable_lsn: durable,
        },
    )
    .await?;
    Ok(())
}

async fn session(inner: &Arc<Inner>, progressed: &mut bool) -> ReplResult<()> {
    let conn = connect(&inner.leader_addr, inner.tls.as_ref()).await?;
    let (mut rd, mut wr) = tokio::io::split(conn);

    let applied = inner.applied.load(Ordering::Acquire);
    let pos_epoch = inner.pos_epoch.load(Ordering::Acquire);
    let hello = match &inner.table {
        Some(table) => Message::HelloTable {
            version: PROTO_VERSION,
            table: table.clone(),
            applied_lsn: applied,
        },
        None => Message::Hello {
            version: PROTO_VERSION,
            applied_lsn: applied,
            epoch: pos_epoch,
            history: inner.history(pos_epoch, applied),
        },
    };
    write_message(&mut wr, &hello).await?;
    let welcome = match recv(inner, &mut rd).await? {
        Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
        m @ Message::Welcome { .. } => m,
        _ => return Err(ReplicationError::Protocol("expected welcome".into())),
    };
    let Message::Welcome {
        snapshot,
        start_lsn,
        tip,
        epoch,
        truncate,
        common_epoch,
        common_lsn,
        ..
    } = welcome
    else {
        unreachable!("matched above")
    };
    if epoch < inner.known_epoch() {
        inner
            .metrics
            .stale_epoch_frames
            .fetch_add(1, Ordering::Relaxed);
        return Err(ReplicationError::Protocol(format!(
            "leader is at stale epoch {epoch} (known {})",
            inner.known_epoch()
        )));
    }
    if let Some(k) = &inner.known {
        k.advance(epoch);
    }
    inner.metrics.connected.store(1, Ordering::Relaxed);
    if epoch != pos_epoch {
        inner.metrics.leader_tip_lsn.store(0, Ordering::Release);
    }
    inner.metrics.observe_tip(tip);
    *progressed = true;

    if snapshot {
        receive_snapshot(inner, &mut rd, epoch).await?;
    } else if truncate {
        truncate_tail(
            inner,
            &mut rd,
            &mut wr,
            epoch,
            start_lsn,
            common_epoch,
            common_lsn,
        )
        .await?;
    } else if pos_epoch != epoch || applied != start_lsn {
        // Same follower state, new coordinates: the new primary mapped our
        // position onto its own WAL. Nothing to rebuild, nothing to truncate.
        inner.set_position(epoch, start_lsn, false).await?;
        inner
            .metrics
            .incremental_resumes
            .fetch_add(1, Ordering::Relaxed);
    }
    let now = inner.applied.load(Ordering::Acquire);
    send_ack(&mut wr, epoch, now, now).await?;

    let mut stamps: HashMap<u64, [u8; 12]> = HashMap::new();
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
            Message::Stamps(st) => stamps.extend(st),
            Message::Frames {
                epoch: fe,
                tip,
                frames,
            } => {
                check_epoch(inner, epoch, fe)?;
                inner.metrics.observe_tip(tip);
                let before = inner.applied.load(Ordering::Acquire);
                let n = frames.len() as u64;
                let engine = Arc::clone(&inner.engine);
                let key = inner.key.clone();
                let st = std::mem::take(&mut stamps);
                let (new_applied, ops) = tokio::task::spawn_blocking(move || {
                    apply_frames(&engine, key.as_deref(), frames, before, &st)
                })
                .await
                .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                if new_applied == before {
                    continue;
                }
                // Tell the leader the frames are applied, then make the
                // position durable before publishing it locally and acking
                // durability.
                send_ack(&mut wr, epoch, new_applied, before).await?;
                inner.set_position(epoch, new_applied, false).await?;
                inner.metrics.frames_applied.fetch_add(n, Ordering::Relaxed);
                inner.metrics.rows_applied.fetch_add(ops, Ordering::Relaxed);
                send_ack(&mut wr, epoch, new_applied, new_applied).await?;
            }
            Message::Heartbeat { epoch: he, tip } => {
                check_epoch(inner, epoch, he)?;
                inner.metrics.observe_tip(tip);
                let a = inner.applied.load(Ordering::Acquire);
                send_ack(&mut wr, epoch, a, a).await?;
            }
            Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
            _ => return Err(ReplicationError::Protocol("unexpected message".into())),
        }
    }
}

/// The follower applied frames the new primary does not have (an un-acked tail
/// of a former primary, or a follower that ran ahead). Keep everything up to
/// the common point, ask the primary for its current value of every key the
/// tail touched, and resume streaming from the common point.
async fn truncate_tail(
    inner: &Arc<Inner>,
    rd: &mut ConnRead,
    wr: &mut ConnWrite,
    epoch: u64,
    start_lsn: u64,
    common_epoch: u64,
    common_lsn: u64,
) -> ReplResult<()> {
    let tail = collect_tail_pks(inner, common_epoch, common_lsn).await;
    write_message(
        wr,
        &Message::TailKeys {
            ok: tail.is_some(),
            pks: tail.clone().unwrap_or_default(),
        },
    )
    .await?;
    let Some(pks) = tail else {
        // The tail cannot be identified: the primary falls back to a snapshot.
        return receive_snapshot(inner, rd, epoch).await;
    };
    let mut seen: HashSet<u64> = HashSet::new();
    loop {
        match recv(inner, rd).await? {
            Message::SnapshotRows(rows) => {
                seen.extend(rows.iter().map(|(pk, _)| *pk));
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
                // Keys the primary no longer has: the tail's inserts.
                let gone: Vec<(u64, Option<Vec<u64>>)> = pks
                    .iter()
                    .filter(|pk| !seen.contains(pk))
                    .map(|pk| (*pk, None))
                    .collect();
                let engine = Arc::clone(&inner.engine);
                let lineage = inner.lineage.clone();
                tokio::task::spawn_blocking(move || -> ReplResult<()> {
                    for chunk in gone.chunks(4096) {
                        engine.apply_batch(chunk.to_vec())?;
                    }
                    if let Some(l) = lineage {
                        l.lock().truncate_after(common_epoch, common_lsn)?;
                    }
                    Ok(())
                })
                .await
                .map_err(|e| ReplicationError::Engine(e.to_string()))??;
                inner
                    .metrics
                    .fix_rows
                    .fetch_add(pks.len() as u64, Ordering::Relaxed);
                inner
                    .set_position(epoch, snapshot_lsn.max(start_lsn), false)
                    .await?;
                inner
                    .metrics
                    .incremental_resumes
                    .fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
            _ => return Err(ReplicationError::Protocol("unexpected in tail fix".into())),
        }
    }
}

/// Keys touched by our own WAL after the common point; `None` if the WAL no
/// longer reaches back that far (or the point is unknown to our lineage).
async fn collect_tail_pks(
    inner: &Arc<Inner>,
    common_epoch: u64,
    common_lsn: u64,
) -> Option<Vec<u64>> {
    let lineage = inner.lineage.clone()?;
    let tip = inner.engine.wal_tip_sequence();
    let mapped = lineage.lock().map(common_epoch, common_lsn, tip)?;
    let wal_dir = inner.data_dir.join("wal");
    let key = inner.key.clone();
    tokio::task::spawn_blocking(move || {
        let b = wal_bounds(&wal_dir).ok()?;
        if b.first_seq.is_some_and(|f| f > mapped.local + 1) {
            return None;
        }
        let mut tail = WalTail::open(&wal_dir, mapped.local).ok()?;
        let mut pks = std::collections::BTreeSet::new();
        loop {
            let batch = tail.read_batch(512).ok()?;
            if batch.is_empty() {
                break;
            }
            for f in batch {
                for (pk, _) in frame_ops(key.as_deref(), f).ok()? {
                    pks.insert(pk);
                }
            }
        }
        Some(pks.into_iter().collect())
    })
    .await
    .ok()?
}

async fn receive_snapshot(inner: &Arc<Inner>, rd: &mut ConnRead, epoch: u64) -> ReplResult<()> {
    // Position 0 *before* wiping: if the session dies half-way, the next one
    // must not resume streaming over a half-built replica.
    let dir = inner.data_dir.clone();
    let engine = Arc::clone(&inner.engine);
    let lineage = inner.lineage.clone();
    tokio::task::spawn_blocking(move || -> ReplResult<()> {
        save_position_full(
            &dir,
            &Position {
                epoch,
                lsn: 0,
                primary: false,
            },
        )?;
        if let Some(l) = lineage {
            l.lock().reset_to(epoch, 0, 0)?;
        }
        wipe(&engine)
    })
    .await
    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
    inner.applied.store(0, Ordering::Release);
    inner.pos_epoch.store(epoch, Ordering::Release);
    inner.metrics.applied_lsn.store(0, Ordering::Release);

    loop {
        match recv(inner, rd).await? {
            Message::SnapshotRows(rows) => {
                inner
                    .metrics
                    .snapshot_rows
                    .fetch_add(rows.len() as u64, Ordering::Relaxed);
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
                inner.set_position(epoch, snapshot_lsn, true).await?;
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
pub(crate) fn wipe(engine: &TableEngine) -> ReplResult<()> {
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

/// Decode one WAL frame into final-state row operations.
fn frame_ops(
    key: Option<&vectorvolt_crypto::KeyStore>,
    f: WireFrame,
) -> ReplResult<Vec<(u64, Option<Vec<u64>>)>> {
    let header = SegmentHeader::with_key_id(0, 0, f.key_id);
    let payload = vectorvolt_txn::manager::open_wal_payload(key, &header, f.seq, f.payload)
        .map_err(|e| ReplicationError::Decrypt(format!("lsn {}: {}", e.seq, e.reason)))?;
    Ok(match Mutation::decode(&payload) {
        Ok(Mutation::Insert { pk, values })
        | Ok(Mutation::Upsert { pk, values })
        | Ok(Mutation::Update { pk, values }) => vec![(pk, Some(values))],
        Ok(Mutation::Delete { pk }) => vec![(pk, None)],
        Ok(Mutation::InsertBatch { rows }) | Ok(Mutation::UpsertBatch { rows }) => {
            rows.into_iter().map(|(pk, v)| (pk, Some(v))).collect()
        }
        Ok(Mutation::DeleteBatch { pks }) => pks.into_iter().map(|pk| (pk, None)).collect(),
        Ok(Mutation::TxnBatch { ops }) => ops,
        // Checkpoints, DDL and non-mutation payloads carry no row data.
        Ok(_) | Err(_) => Vec::new(),
    })
}

/// Decode `frames` the way the engine's WAL replay does and apply them.
/// Returns the new applied LSN and the number of row operations.
fn apply_frames(
    engine: &TableEngine,
    key: Option<&vectorvolt_crypto::KeyStore>,
    frames: Vec<WireFrame>,
    applied: u64,
    stamps: &HashMap<u64, [u8; 12]>,
) -> ReplResult<(u64, u64)> {
    let mut expected = applied + 1;
    let mut ops: Vec<(u64, Option<Vec<u64>>)> = Vec::new();
    let mut ops_stamp: Option<[u8; 12]> = None;
    let mut n_ops = 0u64;
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
        let stamp = stamps.get(&f.seq).copied();
        let fops = frame_ops(key, f)?;
        if fops.is_empty() {
            continue;
        }
        // Frames with different primary stamps are different commits: keep
        // them in separate batches so each event carries its own stamp.
        if !ops.is_empty() && stamp != ops_stamp {
            n_ops += ops.len() as u64;
            flush_ops(engine, std::mem::take(&mut ops), ops_stamp)?;
        }
        ops_stamp = stamp;
        ops.extend(fops);
    }
    n_ops += ops.len() as u64;
    flush_ops(engine, ops, ops_stamp)?;
    Ok((last, n_ops))
}

/// Process-wide hook told about every versionstamp received from a primary,
/// before the commit is applied (the cluster layer merges it into its clock).
/// Callback receiving a primary's 12-byte commit versionstamp.
pub type StampObserver = Arc<dyn Fn(&[u8]) + Send + Sync>;

static STAMP_OBSERVER: OnceLock<StampObserver> = OnceLock::new();

/// Install the stamp observer. Returns `false` if one is already installed.
pub fn set_stamp_observer(f: StampObserver) -> bool {
    STAMP_OBSERVER.set(f).is_ok()
}

fn flush_ops(
    engine: &TableEngine,
    ops: Vec<(u64, Option<Vec<u64>>)>,
    stamp: Option<[u8; 12]>,
) -> ReplResult<()> {
    if ops.is_empty() {
        return Ok(());
    }
    if let (Some(s), Some(obs)) = (stamp, STAMP_OBSERVER.get()) {
        obs(&s);
    }
    // Keep only the last op per pk (ops are final states, order matters
    // per key only), preserving relative order.
    let mut seen = HashSet::new();
    let mut dedup: Vec<_> = ops
        .into_iter()
        .rev()
        .filter(|(pk, _)| seen.insert(*pk))
        .collect();
    dedup.reverse();
    engine.apply_batch_stamped(dedup, stamp)?;
    Ok(())
}
