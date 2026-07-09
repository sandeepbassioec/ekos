//! GroupCommitBatcher — high-throughput write path.
//!
//! Callers submit payloads via `GroupCommitBatcher::submit()` and await a
//! `CommitFuture` that resolves once the payload is durably flushed to disk.
//!
//! The batcher runs a background Tokio task that:
//!   1. Drains the bounded submission channel.
//!   2. Appends all pending frames to the WalWriter (in-memory, no fsync yet).
//!   3. Calls `WalWriter::flush()` once for the entire batch (one fsync for N writes).
//!   4. Notifies all waiters for that batch via oneshot channels.
//!   5. Records flush metrics (count, bytes, latency) into BatcherMetrics.
//!
//! Backpressure: the channel is bounded. `submit()` returns
//! `Err(WalError::Backpressure)` when the channel is full rather than
//! blocking, giving callers the choice to retry, shed load, or surface the
//! error upstream.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Notify, oneshot};
use tokio::time::{Duration, interval};
use tracing::{debug, error, warn};

use crate::error::{WalError, WalResult};
use crate::metrics::BatcherMetrics;
use crate::writer::{WalTip, WalWriter};

type DoneSender = oneshot::Sender<WalResult<WalTip>>;

struct Submission {
    payload: Vec<u8>,
    done_tx: DoneSender,
}

/// Handle returned by `GroupCommitBatcher::submit()`.
/// Await it to know when the payload is durable.
pub struct CommitFuture(oneshot::Receiver<WalResult<WalTip>>);

impl CommitFuture {
    pub async fn wait(self) -> WalResult<WalTip> {
        self.0.await.unwrap_or(Err(WalError::ChannelClosed))
    }
}

/// Configuration for the GroupCommitBatcher.
#[derive(Clone, Debug)]
pub struct BatcherConfig {
    /// Maximum time between forced flushes.
    pub flush_interval: Duration,
    /// Flush early when pending batch exceeds this many bytes.
    pub max_batch_bytes: usize,
    /// Bounded channel depth. When full, `submit()` returns Backpressure.
    pub channel_capacity: usize,
}

impl Default for BatcherConfig {
    fn default() -> Self {
        BatcherConfig {
            flush_interval: Duration::from_micros(500),
            max_batch_bytes: 4 * 1024 * 1024, // 4 MiB
            channel_capacity: 8192,
        }
    }
}

#[derive(Clone)]
pub struct GroupCommitBatcher {
    tx: crossbeam_channel::Sender<Submission>,
    shutdown: Arc<Notify>,
    metrics: BatcherMetrics,
}

impl GroupCommitBatcher {
    /// Spawn the batcher background task with default config.
    pub fn start(
        writer: WalWriter,
        flush_interval: Duration,
        max_batch_bytes: usize,
        channel_capacity: usize,
    ) -> Self {
        Self::start_with_config(
            writer,
            BatcherConfig {
                flush_interval,
                max_batch_bytes,
                channel_capacity,
            },
        )
    }

    /// Spawn the batcher background task with explicit config.
    pub fn start_with_config(writer: WalWriter, cfg: BatcherConfig) -> Self {
        let (tx, rx) = crossbeam_channel::bounded::<Submission>(cfg.channel_capacity);
        let shutdown = Arc::new(Notify::new());
        let metrics = BatcherMetrics::new();

        let shutdown_bg = shutdown.clone();
        let metrics_bg = metrics.clone();
        let max_bytes = cfg.max_batch_bytes;

        tokio::spawn(async move {
            let mut ticker = interval(cfg.flush_interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {},
                    _ = shutdown_bg.notified() => {
                        // Graceful drain: flush whatever is queued.
                        Self::drain_and_flush(&rx, &writer, max_bytes, &metrics_bg);
                        break;
                    }
                }
                Self::drain_and_flush(&rx, &writer, max_bytes, &metrics_bg);
            }
        });

        Self {
            tx,
            shutdown,
            metrics,
        }
    }

    fn drain_and_flush(
        rx: &crossbeam_channel::Receiver<Submission>,
        writer: &WalWriter,
        max_batch_bytes: usize,
        metrics: &BatcherMetrics,
    ) {
        let mut batch: Vec<Submission> = Vec::new();
        let mut batch_bytes = 0usize;

        loop {
            match rx.try_recv() {
                Ok(s) => {
                    batch_bytes += s.payload.len();
                    batch.push(s);
                    if batch_bytes >= max_batch_bytes {
                        break;
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => break,
            }
        }

        if batch.is_empty() {
            return;
        }

        let frame_count = batch.len() as u64;
        debug!(
            count = frame_count,
            bytes = batch_bytes,
            "flushing group commit batch"
        );

        let t0 = Instant::now();

        let mut tips: Vec<WalResult<WalTip>> = batch
            .iter()
            .map(|s| writer.append(s.payload.clone()))
            .collect();

        let flush_result = writer.flush();
        let latency_us = t0.elapsed().as_micros() as u64;

        if let Err(ref e) = flush_result {
            error!(error = %e, "WAL flush failed");
            for (sub, _) in batch.into_iter().zip(tips.drain(..)) {
                let _ = sub
                    .done_tx
                    .send(Err(WalError::Io(std::io::Error::other(e.to_string()))));
            }
            return;
        }

        metrics.record_flush(frame_count, batch_bytes as u64, latency_us);

        for (sub, tip) in batch.into_iter().zip(tips.into_iter()) {
            let _ = sub.done_tx.send(tip);
        }
    }

    /// Submit a payload for durable commit. Returns immediately.
    ///
    /// Returns `Err(WalError::Backpressure)` when the channel is full — the
    /// caller should back off and retry rather than blocking.
    pub fn submit(&self, payload: Vec<u8>) -> WalResult<CommitFuture> {
        let (done_tx, done_rx) = oneshot::channel();
        match self.tx.try_send(Submission { payload, done_tx }) {
            Ok(()) => Ok(CommitFuture(done_rx)),
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                warn!("GroupCommitBatcher channel full — backpressure applied");
                self.metrics.record_drop();
                Err(WalError::Backpressure)
            }
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => Err(WalError::ChannelClosed),
        }
    }

    /// Trigger a graceful shutdown — drains the queue then stops the task.
    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }

    /// Live metrics handle — readable from any thread, zero lock contention.
    pub fn metrics(&self) -> &BatcherMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn concurrent_commits() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg).unwrap();

        let batcher = GroupCommitBatcher::start(
            writer.clone(),
            Duration::from_micros(500),
            4 * 1024 * 1024,
            8192,
        );

        let mut futures = Vec::new();
        for i in 0u32..100 {
            let f = batcher.submit(i.to_le_bytes().to_vec()).unwrap();
            futures.push(f);
        }
        let mut tips = Vec::new();
        for f in futures {
            tips.push(f.wait().await.unwrap());
        }
        let sequences: std::collections::HashSet<u64> = tips.iter().map(|t| t.sequence).collect();
        assert_eq!(sequences.len(), 100);
        assert!(sequences.contains(&1));
        assert!(sequences.contains(&100));
    }

    #[tokio::test]
    async fn metrics_recorded() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg).unwrap();

        let batcher =
            GroupCommitBatcher::start(writer, Duration::from_micros(200), 4 * 1024 * 1024, 8192);

        let n = 50u32;
        let mut futs = Vec::new();
        for i in 0..n {
            futs.push(batcher.submit(i.to_le_bytes().to_vec()).unwrap());
        }
        for f in futs {
            f.wait().await.unwrap();
        }

        let snap = batcher.metrics().snapshot();
        assert_eq!(snap.frames_committed, n as u64);
        assert!(snap.fsync_count >= 1);
        assert_eq!(snap.frames_dropped, 0);
    }

    #[tokio::test]
    async fn backpressure_when_channel_full() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg).unwrap();

        // Tiny channel, slow flush interval so it fills before draining
        let batcher = GroupCommitBatcher::start(
            writer,
            Duration::from_secs(60), // won't flush during test
            4 * 1024 * 1024,
            2, // only 2 slots
        );

        let _f1 = batcher.submit(b"a".to_vec()).unwrap();
        let _f2 = batcher.submit(b"b".to_vec()).unwrap();

        // Third submit should see backpressure
        assert!(matches!(
            batcher.submit(b"c".to_vec()),
            Err(WalError::Backpressure)
        ));
        assert_eq!(batcher.metrics().frames_dropped(), 1);
    }
}
