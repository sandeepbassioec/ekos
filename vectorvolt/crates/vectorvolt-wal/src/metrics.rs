//! BatcherMetrics — lock-free counters for the GroupCommitBatcher.
//!
//! All counters are atomic u64 so they can be read from any thread without
//! taking the writer lock.  Latency is tracked as a simple exponential
//! moving average (EMA) with α = 0.1 — cheap, no heap allocation, good
//! enough for operational dashboards.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Shared metrics handle — cheap to clone (Arc inside).
#[derive(Clone, Default)]
pub struct BatcherMetrics(Arc<Inner>);

#[derive(Default)]
struct Inner {
    /// Number of fsync calls issued by the batcher background task.
    fsync_count: AtomicU64,
    /// Total frames committed across all batches.
    frames_committed: AtomicU64,
    /// Total bytes committed across all batches.
    bytes_committed: AtomicU64,
    /// Number of batches flushed (one fsync per batch).
    batch_count: AtomicU64,
    /// EMA of batch size in frames, stored as fixed-point (value × 1000).
    ema_batch_size_fp: AtomicU64,
    /// EMA of flush latency in microseconds, stored as fixed-point (value × 1000).
    ema_latency_us_fp: AtomicU64,
    /// Peak batch size observed (frames).
    peak_batch_size: AtomicU64,
    /// Total frames dropped due to backpressure (channel full).
    frames_dropped: AtomicU64,
}

const EMA_ALPHA_FP: u64 = 100; // α = 0.1 in fixed-point × 1000

impl BatcherMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called by the background task after each batch flush.
    pub fn record_flush(&self, frame_count: u64, byte_count: u64, latency_us: u64) {
        let inner = &self.0;
        inner.fsync_count.fetch_add(1, Ordering::Relaxed);
        inner.frames_committed.fetch_add(frame_count, Ordering::Relaxed);
        inner.bytes_committed.fetch_add(byte_count, Ordering::Relaxed);
        inner.batch_count.fetch_add(1, Ordering::Relaxed);

        // Update peak
        let mut prev = inner.peak_batch_size.load(Ordering::Relaxed);
        while frame_count > prev {
            match inner.peak_batch_size.compare_exchange_weak(
                prev, frame_count, Ordering::Relaxed, Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(cur) => prev = cur,
            }
        }

        // EMA batch size: new = alpha * sample + (1-alpha) * old
        let sample_fp = frame_count * 1000;
        let old_fp = inner.ema_batch_size_fp.load(Ordering::Relaxed);
        let new_fp = (EMA_ALPHA_FP * sample_fp + (1000 - EMA_ALPHA_FP) * old_fp) / 1000;
        inner.ema_batch_size_fp.store(new_fp, Ordering::Relaxed);

        // EMA latency
        let lat_fp = latency_us * 1000;
        let old_lat = inner.ema_latency_us_fp.load(Ordering::Relaxed);
        let new_lat = (EMA_ALPHA_FP * lat_fp + (1000 - EMA_ALPHA_FP) * old_lat) / 1000;
        inner.ema_latency_us_fp.store(new_lat, Ordering::Relaxed);
    }

    pub fn record_drop(&self) {
        self.0.frames_dropped.fetch_add(1, Ordering::Relaxed);
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn fsync_count(&self) -> u64 {
        self.0.fsync_count.load(Ordering::Relaxed)
    }

    pub fn frames_committed(&self) -> u64 {
        self.0.frames_committed.load(Ordering::Relaxed)
    }

    pub fn bytes_committed(&self) -> u64 {
        self.0.bytes_committed.load(Ordering::Relaxed)
    }

    pub fn batch_count(&self) -> u64 {
        self.0.batch_count.load(Ordering::Relaxed)
    }

    /// Average batch size in frames (from EMA).
    pub fn avg_batch_size(&self) -> f64 {
        self.0.ema_batch_size_fp.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Average flush latency in microseconds (from EMA).
    pub fn avg_latency_us(&self) -> f64 {
        self.0.ema_latency_us_fp.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn peak_batch_size(&self) -> u64 {
        self.0.peak_batch_size.load(Ordering::Relaxed)
    }

    pub fn frames_dropped(&self) -> u64 {
        self.0.frames_dropped.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            fsync_count:     self.fsync_count(),
            frames_committed: self.frames_committed(),
            bytes_committed: self.bytes_committed(),
            batch_count:     self.batch_count(),
            avg_batch_size:  self.avg_batch_size(),
            avg_latency_us:  self.avg_latency_us(),
            peak_batch_size: self.peak_batch_size(),
            frames_dropped:  self.frames_dropped(),
        }
    }
}

/// Point-in-time snapshot of all batcher metrics.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub fsync_count:      u64,
    pub frames_committed: u64,
    pub bytes_committed:  u64,
    pub batch_count:      u64,
    pub avg_batch_size:   f64,
    pub avg_latency_us:   f64,
    pub peak_batch_size:  u64,
    pub frames_dropped:   u64,
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fsyncs={} batches={} frames={} bytes={} avg_batch={:.1} avg_lat={:.1}µs peak={} dropped={}",
            self.fsync_count,
            self.batch_count,
            self.frames_committed,
            self.bytes_committed,
            self.avg_batch_size,
            self.avg_latency_us,
            self.peak_batch_size,
            self.frames_dropped,
        )
    }
}
