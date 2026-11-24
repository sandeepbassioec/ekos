//! Additive engine hooks used by the cluster layer.
//!
//! * [`CommitStamper`] (`TableEngine::set_commit_stamper`): called once per
//!   commit with the CDC LSN; the returned 12-byte versionstamp is stored on
//!   the [`ChangeEvent`](crate::cdc::ChangeEvent) so CDC consumers and sinks see
//!   it. Hot-path cost with no stamper installed: one relaxed atomic load.
//! * [`WriteGuard`] (`TableEngine::set_write_guard`): the engine's per-key
//!   write gate. Every mutation entry point asks the guard first, so shard
//!   freeze/ownership is enforced for *all* listeners (wire, HTTP, gRPC,
//!   pgwire, SQL) at once. An in-flight counter lets a caller that has just
//!   flipped the guard wait until every write that already passed it has
//!   finished ([`TableEngine::quiesce_writes`](crate::TableEngine::quiesce_writes)).

use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// `lsn -> versionstamp`; `None` leaves the event unstamped.
pub type CommitStamper = Arc<dyn Fn(u64) -> Option<[u8; 12]> + Send + Sync>;
/// `pk -> Ok` to admit the write or `Err(reason)` to reject it.
pub type WriteGuard = Arc<dyn Fn(u64) -> Result<(), String> + Send + Sync>;

#[derive(Default)]
pub(crate) struct EngineHooks {
    stamper_on: AtomicBool,
    stamper: RwLock<Option<CommitStamper>>,
    guard_on: AtomicBool,
    guard: RwLock<Option<WriteGuard>>,
    inflight: AtomicU64,
    /// Stamped commits whose WAL frame may be visible but whose stamp is not
    /// yet in `ring` (see [`EngineHooks::pending`]).
    pending: AtomicU64,
    /// `WAL sequence -> versionstamp` of recent commits, so replication can
    /// ship the primary's stamp with each frame.
    ring: parking_lot::Mutex<std::collections::BTreeMap<u64, [u8; 12]>>,
}

/// Bound on the in-memory stamp ring.
const RING_CAP: usize = 262_144;

/// Marks a stamped commit in flight (see [`EngineHooks::pending`]).
pub(crate) struct PendingGuard<'a>(Option<&'a AtomicU64>);

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        if let Some(c) = self.0 {
            c.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// Held for the duration of one guarded write; releases the in-flight count.
pub(crate) struct WriteTicket<'a>(Option<&'a AtomicU64>);

impl Drop for WriteTicket<'_> {
    fn drop(&mut self) {
        if let Some(c) = self.0 {
            c.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl EngineHooks {
    pub(crate) fn set_stamper(&self, s: Option<CommitStamper>) {
        let on = s.is_some();
        *self.stamper.write() = s;
        self.stamper_on.store(on, Ordering::Release);
    }

    pub(crate) fn set_guard(&self, g: Option<WriteGuard>) {
        let on = g.is_some();
        *self.guard.write() = g;
        self.guard_on.store(on, Ordering::SeqCst);
    }

    /// One stamp for the commit whose first event has `lsn`.
    #[inline]
    pub(crate) fn stamp(&self, lsn: u64) -> Option<[u8; 12]> {
        if !self.stamper_on.load(Ordering::Acquire) {
            return None;
        }
        self.stamper.read().as_ref().and_then(|s| s(lsn))
    }

    /// Held across a commit so replication can wait for its stamp to be
    /// recorded. No-op when no stamper is installed.
    #[inline]
    pub(crate) fn pending(&self) -> PendingGuard<'_> {
        if !self.stamper_on.load(Ordering::Acquire) {
            return PendingGuard(None);
        }
        self.pending.fetch_add(1, Ordering::SeqCst);
        PendingGuard(Some(&self.pending))
    }

    pub(crate) fn pending_count(&self) -> u64 {
        self.pending.load(Ordering::SeqCst)
    }

    /// Remember the stamp of the commit whose WAL frame has sequence `seq`.
    pub(crate) fn record(&self, seq: u64, stamp: Option<[u8; 12]>) {
        let Some(st) = stamp else { return };
        if seq == 0 {
            return;
        }
        let mut r = self.ring.lock();
        r.insert(seq, st);
        while r.len() > RING_CAP {
            r.pop_first();
        }
    }

    pub(crate) fn stamps_in(&self, lo: u64, hi: u64) -> Vec<(u64, [u8; 12])> {
        if lo > hi {
            return Vec::new();
        }
        self.ring.lock().range(lo..=hi).map(|(k, v)| (*k, *v)).collect()
    }

    /// Admit a write to every key in `pks` or reject the lot.
    #[inline]
    pub(crate) fn enter(
        &self,
        pks: impl IntoIterator<Item = u64>,
    ) -> Result<WriteTicket<'_>, std::io::Error> {
        if !self.guard_on.load(Ordering::SeqCst) {
            return Ok(WriteTicket(None));
        }
        // Count first, then check: a concurrent freeze either rejects us or
        // sees `inflight > 0` and waits for us (see `quiesce`).
        self.inflight.fetch_add(1, Ordering::SeqCst);
        let ticket = WriteTicket(Some(&self.inflight));
        if let Some(g) = self.guard.read().as_ref() {
            for pk in pks {
                g(pk).map_err(|why| {
                    std::io::Error::new(std::io::ErrorKind::PermissionDenied, why)
                })?;
            }
        }
        Ok(ticket)
    }

    /// Wait until every write admitted so far has finished. Call after
    /// flipping the guard to reject.
    pub(crate) fn quiesce(&self, timeout: std::time::Duration) -> bool {
        let end = std::time::Instant::now() + timeout;
        while self.inflight.load(Ordering::SeqCst) != 0 {
            if std::time::Instant::now() > end {
                return false;
            }
            std::thread::yield_now();
        }
        true
    }
}
