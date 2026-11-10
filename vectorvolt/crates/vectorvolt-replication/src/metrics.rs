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
        }
    }
}
