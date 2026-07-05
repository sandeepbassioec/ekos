//! GroupCommitBatcher — high-throughput write path.
//!
//! Callers submit payloads via `GroupCommitBatcher::submit()` and await a
//! `CommitFuture` that resolves once the payload is durably flushed to disk.
//!
//! The batcher runs a background Tokio task that:
//!   1. Drains the submission channel.
//!   2. Appends all pending frames to the WalWriter (in-memory, no fsync yet).
//!   3. Calls `WalWriter::flush()` once for the entire batch (one fsync for N writes).
//!   4. Notifies all waiters for that batch.
//!
//! This turns N sequential fsyncs into 1, which is the primary throughput lever
//! for write-heavy workloads.

use crate::error::{WalError, WalResult};
use crate::writer::{WalTip, WalWriter};
use std::sync::Arc;
use tokio::sync::{oneshot, Notify};
use tokio::time::{interval, Duration};
use tracing::{debug, error};

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

#[derive(Clone)]
pub struct GroupCommitBatcher {
    tx:       crossbeam_channel::Sender<Submission>,
    shutdown: Arc<Notify>,
}

impl GroupCommitBatcher {
    /// Spawn the batcher background task.
    ///
    /// `flush_interval` — maximum time between flushes.
    /// `max_batch_bytes` — flush early when the pending batch exceeds this size.
    pub fn start(
        writer: WalWriter,
        flush_interval: Duration,
        max_batch_bytes: usize,
        channel_capacity: usize,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::bounded::<Submission>(channel_capacity);
        let shutdown = Arc::new(Notify::new());
        let shutdown_clone = shutdown.clone();

        tokio::spawn(async move {
            let mut ticker = interval(flush_interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                // Wait for either the flush interval or a shutdown signal.
                tokio::select! {
                    _ = ticker.tick() => {},
                    _ = shutdown_clone.notified() => {
                        // Drain remaining submissions before exiting.
                        Self::drain_and_flush(&rx, &writer, max_batch_bytes);
                        break;
                    }
                }
                Self::drain_and_flush(&rx, &writer, max_batch_bytes);
            }
        });

        Self { tx, shutdown }
    }

    fn drain_and_flush(
        rx: &crossbeam_channel::Receiver<Submission>,
        writer: &WalWriter,
        max_batch_bytes: usize,
    ) {
        let mut batch: Vec<Submission> = Vec::new();
        let mut batch_bytes = 0usize;

        // Drain everything currently in the channel (non-blocking).
        loop {
            match rx.try_recv() {
                Ok(s) => {
                    batch_bytes += s.payload.len();
                    batch.push(s);
                    if batch_bytes >= max_batch_bytes { break; }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => break,
            }
        }

        if batch.is_empty() { return; }

        debug!(count = batch.len(), bytes = batch_bytes, "flushing group commit batch");

        // Append all frames, then flush once.
        let mut tips: Vec<WalResult<WalTip>> = batch.iter()
            .map(|s| writer.append(s.payload.clone()))
            .collect();

        let flush_result = writer.flush();

        // If the flush itself failed, propagate to all waiters.
        if let Err(e) = flush_result {
            error!(error = %e, "WAL flush failed");
            for (sub, _) in batch.into_iter().zip(tips.drain(..)) {
                let _ = sub.done_tx.send(Err(WalError::Io(
                    std::io::Error::other(e.to_string())
                )));
            }
            return;
        }

        for (sub, tip) in batch.into_iter().zip(tips.into_iter()) {
            let _ = sub.done_tx.send(tip);
        }
    }

    /// Submit a payload for durable commit. Returns immediately; await the
    /// `CommitFuture` to block until the payload is flushed to disk.
    pub fn submit(&self, payload: Vec<u8>) -> WalResult<CommitFuture> {
        let (done_tx, done_rx) = oneshot::channel();
        self.tx.send(Submission { payload, done_tx })
            .map_err(|_| WalError::ChannelClosed)?;
        Ok(CommitFuture(done_rx))
    }

    /// Flush the batcher immediately (useful for graceful shutdown).
    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalConfig;
    use crate::writer::WalWriter;
    use tempfile::TempDir;
    use tokio::time::Duration;

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

        // All 100 writes committed; sequences are 1..=100
        let sequences: std::collections::HashSet<u64> = tips.iter().map(|t| t.sequence).collect();
        assert_eq!(sequences.len(), 100);
        assert!(sequences.contains(&1));
        assert!(sequences.contains(&100));
    }
}
