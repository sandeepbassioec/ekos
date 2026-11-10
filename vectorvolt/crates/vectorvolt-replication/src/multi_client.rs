//! [`MultiReplicationClient`] — follower for every table of a leader's registry.
//!
//! Keeps one catalog connection to a `MultiReplicationServer`; each table it
//! announces gets a local table (created with the leader's schema when
//! missing) and its own [`ReplicationClient`] stream with a per-table position
//! file at `<table dir>/replication.pos`. Tables created on the leader later
//! appear on the follower; tables dropped on the leader are dropped here.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use vectorvolt_engine::{TableRegistry, TableSchema};

use crate::client::{connect, save_position, wipe, ReplicationClient, POSITION_FILE};
use crate::config::{ReplicationConfig, ReplicationRole, RoleCell};
use crate::error::{ReplResult, ReplicationError};
use crate::metrics::ReplicationMetrics;
use crate::proto::{read_message, write_message, CatalogEntry, Message, PROTO_VERSION};
use crate::server::{read_tid, write_tid, TID_FILE};
use crate::tls::ReplicationTlsClient;

struct MultiInner {
    registry: Arc<TableRegistry>,
    cfg: ReplicationConfig,
    role: RoleCell,
    leader_addr: String,
    tls: Option<ReplicationTlsClient>,
    tables: Mutex<HashMap<String, Arc<ReplicationClient>>>,
    connected: AtomicBool,
    shutdown: AtomicBool,
    shutdown_notify: Notify,
    last_error: Mutex<Option<String>>,
}

/// Follower for every table of a leader's registry (see the module docs).
pub struct MultiReplicationClient {
    inner: Arc<MultiInner>,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl MultiReplicationClient {
    /// Start replicating every table of the leader at `leader_addr` into
    /// `registry` (whose encryption key, if any, decrypts shipped WAL frames).
    pub fn start(
        registry: Arc<TableRegistry>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: impl Into<String>,
    ) -> ReplResult<Arc<Self>> {
        Self::start_tls(registry, cfg, role, leader_addr, None)
    }

    /// [`start`](Self::start) over TLS / mTLS when `tls` is set.
    pub fn start_tls(
        registry: Arc<TableRegistry>,
        cfg: ReplicationConfig,
        role: RoleCell,
        leader_addr: impl Into<String>,
        tls: Option<ReplicationTlsClient>,
    ) -> ReplResult<Arc<Self>> {
        cfg.validate()?;
        let inner = Arc::new(MultiInner {
            registry,
            cfg,
            role,
            leader_addr: leader_addr.into(),
            tls,
            tables: Mutex::new(HashMap::new()),
            connected: AtomicBool::new(false),
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

    /// Names of the tables currently being replicated.
    pub fn tables(&self) -> Vec<String> {
        let mut v: Vec<String> = self.inner.tables.lock().keys().cloned().collect();
        v.sort();
        v
    }

    /// Highest leader LSN applied for `table`, if it is replicated.
    pub fn applied_lsn(&self, table: &str) -> Option<u64> {
        self.inner.tables.lock().get(table).map(|c| c.applied_lsn())
    }

    /// Metrics of one table's stream.
    pub fn table_metrics(&self, table: &str) -> Option<Arc<ReplicationMetrics>> {
        self.inner.tables.lock().get(table).map(|c| c.metrics())
    }

    /// Wait until `table` exists locally and has applied `lsn`; `false` on timeout.
    pub async fn wait_for_lsn(&self, table: &str, lsn: u64, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if self.applied_lsn(table).is_some_and(|a| a >= lsn) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    /// Whether the catalog connection is currently up.
    pub fn is_connected(&self) -> bool {
        self.inner.connected.load(Ordering::Relaxed)
    }

    /// The most recent catalog-session error, if any (diagnostics).
    pub fn last_error(&self) -> Option<String> {
        self.inner.last_error.lock().clone()
    }

    /// Stop replicating every table. Local data and positions stay intact.
    pub fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        self.inner.shutdown_notify.notify_waiters();
        if let Some(h) = self.task.lock().take() {
            h.abort();
        }
        for c in self.inner.tables.lock().values() {
            c.shutdown();
        }
        self.inner.connected.store(false, Ordering::Relaxed);
    }
}

impl Drop for MultiReplicationClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn run_loop(inner: Arc<MultiInner>) {
    let mut backoff = inner.cfg.backoff_initial;
    loop {
        if inner.shutdown.load(Ordering::Acquire) {
            return;
        }
        if inner.role.get() != ReplicationRole::Follower {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                _ = inner.shutdown_notify.notified() => {}
            }
            continue;
        }
        let mut progressed = false;
        if let Err(e) = catalog_session(&inner, &mut progressed).await {
            tracing::debug!("catalog session ended: {e}");
            *inner.last_error.lock() = Some(e.to_string());
        }
        inner.connected.store(false, Ordering::Relaxed);
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

async fn catalog_session(inner: &Arc<MultiInner>, progressed: &mut bool) -> ReplResult<()> {
    let conn = connect(&inner.leader_addr, inner.tls.as_ref()).await?;
    let (mut rd, mut wr) = tokio::io::split(conn);
    write_message(
        &mut wr,
        &Message::CatalogHello {
            version: PROTO_VERSION,
        },
    )
    .await?;
    inner.connected.store(true, Ordering::Relaxed);
    loop {
        if inner.shutdown.load(Ordering::Acquire) || inner.role.get() != ReplicationRole::Follower {
            return Ok(());
        }
        let (msg, _) = tokio::time::timeout(inner.cfg.read_timeout, read_message(&mut rd))
            .await
            .map_err(|_| ReplicationError::Protocol("leader read timeout".into()))??;
        if inner.role.get() != ReplicationRole::Follower {
            return Ok(());
        }
        match msg {
            Message::Catalog(entries) => {
                *progressed = true;
                let i = Arc::clone(inner);
                tokio::task::spawn_blocking(move || apply_catalog(&i, entries))
                    .await
                    .map_err(|e| ReplicationError::Engine(e.to_string()))??;
            }
            Message::Heartbeat { .. } => {}
            Message::Error(e) => return Err(ReplicationError::Protocol(format!("leader: {e}"))),
            _ => {
                return Err(ReplicationError::Protocol(
                    "unexpected catalog message".into(),
                ))
            }
        }
    }
}

/// Make the local registry match the leader's table list and (re)start the
/// per-table streams. Runs on a blocking thread inside the tokio runtime.
fn apply_catalog(inner: &Arc<MultiInner>, entries: Vec<CatalogEntry>) -> ReplResult<()> {
    let reg = &inner.registry;
    let base = reg.base_dir().to_owned();
    let want: HashSet<&str> = entries.iter().map(|e| e.name.as_str()).collect();

    // Drop replicated tables the leader no longer has. A local table counts as
    // replicated when it carries a `replication.tid` file.
    for name in reg.list_tables() {
        if want.contains(name.as_str()) || read_tid(&base.join(&name)).is_none() {
            continue;
        }
        drop_local(inner, &name);
    }

    for e in entries {
        let dir = base.join(&e.name);
        let local_tid = read_tid(&dir);
        if reg.contains(&e.name) && local_tid.is_some_and(|t| t != e.id) {
            // Same name, different incarnation: the leader dropped and
            // re-created it.
            drop_local(inner, &e.name);
        }
        if !reg.contains(&e.name) {
            let schema = TableSchema::from_text(&e.schema)?;
            let had_dir = dir.join("wal").exists();
            let engine = reg.create_table_typed(&e.name, schema)?;
            if had_dir {
                // The directory still holds the old incarnation's WAL, which
                // the engine just replayed.
                wipe(&engine)?;
            }
            save_position(&dir, 0)?;
            write_tid(&dir, e.id)?;
        } else if local_tid.is_none() {
            write_tid(&dir, e.id)?;
        }
        if inner.tables.lock().contains_key(&e.name) {
            continue;
        }
        let engine = reg.get(&e.name)?;
        let client = ReplicationClient::spawn(
            engine,
            dir,
            reg.encryption_key().cloned(),
            inner.cfg.clone(),
            inner.role.clone(),
            inner.leader_addr.clone(),
            inner.tls.clone(),
            Some(e.name.clone()),
        )?;
        inner.tables.lock().insert(e.name, client);
    }
    Ok(())
}

/// Stop replicating `name` and remove its local copy.
fn drop_local(inner: &MultiInner, name: &str) {
    if let Some(c) = inner.tables.lock().remove(name) {
        c.shutdown();
    }
    let dir = inner.registry.base_dir().join(name);
    let _ = std::fs::remove_file(dir.join(POSITION_FILE));
    let _ = std::fs::remove_file(dir.join(TID_FILE));
    if let Err(e) = inner.registry.drop_table(name) {
        tracing::warn!(table = name, "drop replicated table: {e}");
    }
    // Best effort: the engine may still be open (and, on some platforms, its
    // files locked); the next create wipes any leftovers.
    let _ = std::fs::remove_dir_all(&dir);
}
