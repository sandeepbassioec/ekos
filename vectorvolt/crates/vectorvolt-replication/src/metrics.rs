//! Replication lag and throughput metrics.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Counters shared by the leader server and the follower client.
#[derive(Debug)]
pub struct ReplicationMetrics {
    epoch: Instant,
    pub(crate) frames_sent: AtomicU64,
    pub(crate) frames_applied: AtomicU64,
    pub(crate) bytes_sent: AtomicU64,
    pub(crate) bytes_received: AtomicU64,
    pub(crate) snapshots_sent: AtomicU64,
    pub(crate) snapshots_applied: AtomicU64,
    pub(crate) reconnects: AtomicU64,
    pub(crate) rejected_frames: AtomicU64,
    pub(crate) ack_timeouts: AtomicU64,
    /// Follower: highest leader LSN applied.
    pub(crate) applied_lsn: AtomicU64,
    /// Follower: highest leader LSN known (from frames/heartbeats).
    pub(crate) leader_tip_lsn: AtomicU64,
    /// Follower: 1 + ms since `epoch` when it fell behind; 0 = caught up.
    behind_since_ms: AtomicU64,
    pub(crate) connected: AtomicU64,
    /// Follower: rows received in full snapshots.
    pub(crate) snapshot_rows: AtomicU64,
    /// Follower: row operations applied from streamed frames.
    pub(crate) rows_applied: AtomicU64,
    /// Follower: rows rewritten/removed to truncate a divergent tail.
    pub(crate) fix_rows: AtomicU64,
    /// Follower: sessions that resumed from a mapped point (no snapshot).
    pub(crate) incremental_resumes: AtomicU64,
    /// Follower: frames/heartbeats dropped for carrying a stale epoch.
    pub(crate) stale_epoch_frames: AtomicU64,
    /// Leader: acks dropped for carrying another epoch.
    pub(crate) stale_acks: AtomicU64,
    /// Leader: writes admitted with fewer acks than configured.
    pub(crate) degraded_writes: AtomicU64,
    /// Leader gauge: 1 while running below the configured replica count.
    pub(crate) degraded: AtomicU64,
}

impl Default for ReplicationMetrics {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            frames_sent: AtomicU64::new(0),
            frames_applied: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            snapshots_sent: AtomicU64::new(0),
            snapshots_applied: AtomicU64::new(0),
            reconnects: AtomicU64::new(0),
            rejected_frames: AtomicU64::new(0),
            ack_timeouts: AtomicU64::new(0),
            applied_lsn: AtomicU64::new(0),
            leader_tip_lsn: AtomicU64::new(0),
            behind_since_ms: AtomicU64::new(0),
            connected: AtomicU64::new(0),
            snapshot_rows: AtomicU64::new(0),
            rows_applied: AtomicU64::new(0),
            fix_rows: AtomicU64::new(0),
            incremental_resumes: AtomicU64::new(0),
            stale_epoch_frames: AtomicU64::new(0),
            stale_acks: AtomicU64::new(0),
            degraded_writes: AtomicU64::new(0),
            degraded: AtomicU64::new(0),
        }
    }
}

/// Point-in-time copy of the counters plus derived lag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub frames_sent: u64,
    pub frames_applied: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub snapshots_sent: u64,
    pub snapshots_applied: u64,
    pub reconnects: u64,
    pub rejected_frames: u64,
    pub ack_timeouts: u64,
    pub applied_lsn: u64,
    pub leader_tip_lsn: u64,
    /// `leader_tip_lsn - applied_lsn` (follower side).
    pub lag_lsn: u64,
    /// Milliseconds the follower has been continuously behind (0 = caught up).
    pub lag_ms: u64,
    pub connected: bool,
    pub snapshot_rows: u64,
    pub rows_applied: u64,
    pub fix_rows: u64,
    pub incremental_resumes: u64,
    pub stale_epoch_frames: u64,
    pub stale_acks: u64,
    pub degraded_writes: u64,
    /// True while the leader runs below the configured replica count.
    pub degraded: bool,
}

impl ReplicationMetrics {
    pub(crate) fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// Re-evaluate caught-up state after applied/tip changed.
    pub(crate) fn update_lag_state(&self) {
        let applied = self.applied_lsn.load(Ordering::Acquire);
        let tip = self.leader_tip_lsn.load(Ordering::Acquire);
        if applied >= tip {
            self.behind_since_ms.store(0, Ordering::Release);
        } else {
            let _ = self.behind_since_ms.compare_exchange(
                0,
                self.now_ms() + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
        }
    }

    pub(crate) fn observe_tip(&self, tip: u64) {
        self.leader_tip_lsn.fetch_max(tip, Ordering::AcqRel);
        self.update_lag_state();
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let applied = self.applied_lsn.load(Ordering::Acquire);
        let tip = self.leader_tip_lsn.load(Ordering::Acquire);
        let behind = self.behind_since_ms.load(Ordering::Acquire);
        MetricsSnapshot {
            frames_sent: self.frames_sent.load(Ordering::Relaxed),
            frames_applied: self.frames_applied.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            snapshots_sent: self.snapshots_sent.load(Ordering::Relaxed),
            snapshots_applied: self.snapshots_applied.load(Ordering::Relaxed),
            reconnects: self.reconnects.load(Ordering::Relaxed),
            rejected_frames: self.rejected_frames.load(Ordering::Relaxed),
            ack_timeouts: self.ack_timeouts.load(Ordering::Relaxed),
            applied_lsn: applied,
            leader_tip_lsn: tip,
            lag_lsn: tip.saturating_sub(applied),
            lag_ms: if behind == 0 {
                0
            } else {
                self.now_ms().saturating_sub(behind - 1)
            },
            connected: self.connected.load(Ordering::Relaxed) != 0,
            snapshot_rows: self.snapshot_rows.load(Ordering::Relaxed),
            rows_applied: self.rows_applied.load(Ordering::Relaxed),
            fix_rows: self.fix_rows.load(Ordering::Relaxed),
            incremental_resumes: self.incremental_resumes.load(Ordering::Relaxed),
            stale_epoch_frames: self.stale_epoch_frames.load(Ordering::Relaxed),
            stale_acks: self.stale_acks.load(Ordering::Relaxed),
            degraded_writes: self.degraded_writes.load(Ordering::Relaxed),
            degraded: self.degraded.load(Ordering::Relaxed) != 0,
        }
    }
}
