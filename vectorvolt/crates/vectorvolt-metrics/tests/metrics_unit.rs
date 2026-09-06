//! Unit tests for vectorvolt-metrics (VV-OBS-005).

use std::sync::Arc;
use std::sync::atomic::Ordering;
use vectorvolt_metrics::{MetricsRegistry, MetricsServer};

#[test]
fn default_registry_all_zero() {
    let r = MetricsRegistry::default();
    assert_eq!(r.connections_accepted.load(Ordering::Relaxed), 0);
    assert_eq!(r.connections_active.load(Ordering::Relaxed), 0);
    assert_eq!(r.bytes_received.load(Ordering::Relaxed), 0);
}

#[test]
fn prometheus_text_contains_all_metric_names() {
    let r = MetricsRegistry::default();
    let text = r.prometheus_text();
    assert!(text.contains("vv_connections_accepted_total"));
    assert!(text.contains("vv_connections_active"));
    assert!(text.contains("vv_bytes_received_total"));
    assert!(text.contains("vv_bytes_sent_total"));
    assert!(text.contains("vv_requests_total"));
    assert!(text.contains("vv_wal_entries_total"));
    assert!(text.contains("vv_errors_total"));
}

#[test]
fn prometheus_text_reflects_incremented_values() {
    let r = MetricsRegistry::default();
    r.connections_accepted.fetch_add(42, Ordering::Relaxed);
    r.bytes_received.fetch_add(1024, Ordering::Relaxed);
    let text = r.prometheus_text();
    assert!(text.contains("vv_connections_accepted_total 42"));
    assert!(text.contains("vv_bytes_received_total 1024"));
}

#[test]
fn prometheus_text_has_help_and_type_lines() {
    let r = MetricsRegistry::default();
    let text = r.prometheus_text();
    assert!(text.contains("# HELP"));
    assert!(text.contains("# TYPE"));
}

#[test]
fn counter_type_annotation_present() {
    let r = MetricsRegistry::default();
    let text = r.prometheus_text();
    assert!(text.contains("# TYPE vv_connections_accepted_total counter"));
}

#[test]
fn gauge_type_annotation_present() {
    let r = MetricsRegistry::default();
    let text = r.prometheus_text();
    assert!(text.contains("# TYPE vv_connections_active gauge"));
}

#[tokio::test]
async fn metrics_server_serves_200_on_metrics_path() {
    let reg = Arc::new(MetricsRegistry::default());
    reg.connections_accepted.fetch_add(7, Ordering::Relaxed);

    let port = 19100u16;
    let srv = MetricsServer::new(reg.clone());
    tokio::spawn(async move { srv.serve(port).await });
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/metrics"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("vv_connections_accepted_total 7"));
}

#[tokio::test]
async fn metrics_server_returns_404_on_other_path() {
    let reg = Arc::new(MetricsRegistry::default());
    let port = 19101u16;
    let srv = MetricsServer::new(reg);
    tokio::spawn(async move { srv.serve(port).await });
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/foo"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
