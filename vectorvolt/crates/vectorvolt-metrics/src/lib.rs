//! Prometheus-compatible metrics registry and pull-based HTTP endpoint (VV-OBS-005).
//!
//! # Quick start
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use vectorvolt_metrics::{MetricsRegistry, MetricsServer};
//!
//! #[tokio::main]
//! async fn main() {
//!     let reg = Arc::new(MetricsRegistry::default());
//!     let srv = MetricsServer::new(reg.clone());
//!     tokio::spawn(async move { srv.serve(9100).await });
//!
//!     reg.connections_accepted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//! }
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// All metrics tracked by VectorVolt.
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    pub connections_accepted: AtomicU64,
    pub connections_active: AtomicU64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub requests_total: AtomicU64,
    pub wal_entries_total: AtomicU64,
    pub errors_total: AtomicU64,
}

impl MetricsRegistry {
    /// Serialize in Prometheus text exposition format (no timestamp).
    pub fn prometheus_text(&self) -> String {
        let mut out = String::with_capacity(512);

        push_counter(
            &mut out,
            "vv_connections_accepted_total",
            "Total TCP connections accepted",
            self.connections_accepted.load(Ordering::Relaxed),
        );
        push_gauge(
            &mut out,
            "vv_connections_active",
            "Currently open connections",
            self.connections_active.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "vv_bytes_received_total",
            "Total bytes received from clients",
            self.bytes_received.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "vv_bytes_sent_total",
            "Total bytes sent to clients",
            self.bytes_sent.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "vv_requests_total",
            "Total requests processed",
            self.requests_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "vv_wal_entries_total",
            "Total WAL entries committed",
            self.wal_entries_total.load(Ordering::Relaxed),
        );
        push_counter(
            &mut out,
            "vv_errors_total",
            "Total errors encountered",
            self.errors_total.load(Ordering::Relaxed),
        );

        out
    }
}

fn push_counter(out: &mut String, name: &str, help: &str, val: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} counter\n{name} {val}\n"
    ));
}

fn push_gauge(out: &mut String, name: &str, help: &str, val: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {val}\n"
    ));
}

/// Minimal tokio TCP server that serves `/metrics` on the given port.
/// All other paths return 404. Bind errors are logged and the server exits.
pub struct MetricsServer {
    registry: Arc<MetricsRegistry>,
}

impl MetricsServer {
    pub fn new(registry: Arc<MetricsRegistry>) -> Self {
        Self { registry }
    }

    /// Bind and serve forever. Intended to be spawned with `tokio::spawn`.
    pub async fn serve(self, port: u16) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let addr = format!("0.0.0.0:{port}");
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                tracing::info!(addr = %addr, "metrics endpoint listening");
                l
            }
            Err(e) => {
                tracing::error!(addr = %addr, err = %e, "failed to bind metrics endpoint");
                return;
            }
        };

        loop {
            let (mut stream, _peer) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(err = %e, "metrics accept error");
                    continue;
                }
            };

            let body = self.registry.prometheus_text();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf).await;

                let is_metrics = buf.starts_with(b"GET /metrics");
                let (status, content_type) = if is_metrics {
                    ("200 OK", "text/plain; version=0.0.4; charset=utf-8")
                } else {
                    ("404 Not Found", "text/plain")
                };
                let body = if is_metrics {
                    body
                } else {
                    "404 Not Found\n".to_string()
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    }
}
