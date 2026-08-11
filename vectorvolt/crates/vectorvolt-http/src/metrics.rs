use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Lightweight atomic counters for the HTTP gateway.
///
/// These are exposed on `GET /metrics` in Prometheus text format and on
/// `GET /health` as a summary.
#[derive(Default)]
pub struct Metrics {
    pub requests_total: AtomicU64,
    pub requests_ok: AtomicU64,
    pub requests_err: AtomicU64,
    pub rows_inserted: AtomicU64,
    pub rows_deleted: AtomicU64,
    pub rows_fetched: AtomicU64,
    pub scans_executed: AtomicU64,
    pub secondary_lookups: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_ok(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_ok.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_err(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_err.fetch_add(1, Ordering::Relaxed);
    }
}
