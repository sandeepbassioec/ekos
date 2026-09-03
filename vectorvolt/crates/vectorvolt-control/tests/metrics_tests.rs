//! Metrics tests (VV-CTL-007).
use std::sync::atomic::Ordering;
use vectorvolt_control::metrics::Metrics;

#[test]
fn test_new_all_zeros() {
    let m = Metrics::new();
    assert_eq!(m.inserts_total.load(Ordering::Relaxed), 0);
    assert_eq!(m.deletes_total.load(Ordering::Relaxed), 0);
    assert_eq!(m.queries_total.load(Ordering::Relaxed), 0);
    assert_eq!(m.bytes_written.load(Ordering::Relaxed), 0);
    assert_eq!(m.active_connections.load(Ordering::Relaxed), 0);
    assert_eq!(m.raft_term.load(Ordering::Relaxed), 0);
    assert_eq!(m.shard_count.load(Ordering::Relaxed), 0);
}

#[test]
fn test_increment_inserts() {
    let m = Metrics::new();
    m.inserts_total.fetch_add(10, Ordering::Relaxed);
    assert_eq!(m.inserts_total.load(Ordering::Relaxed), 10);
}

#[test]
fn test_prometheus_has_help() {
    let m = Metrics::new();
    let text = m.to_prometheus_text();
    assert!(text.contains("# HELP"));
}

#[test]
fn test_prometheus_has_type() {
    let m = Metrics::new();
    let text = m.to_prometheus_text();
    assert!(text.contains("# TYPE"));
}

#[test]
fn test_prometheus_all_7_counters() {
    let m = Metrics::new();
    let text = m.to_prometheus_text();
    let count = text.matches("# HELP").count();
    assert_eq!(count, 7);
}

#[test]
fn test_prometheus_value_reflected() {
    let m = Metrics::new();
    m.raft_term.store(99, Ordering::Relaxed);
    let text = m.to_prometheus_text();
    assert!(text.contains("vv_raft_term 99"));
}

#[test]
fn test_json_valid() {
    let m = Metrics::new();
    let j = m.to_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v.is_object());
}

#[test]
fn test_json_inserts_field() {
    let m = Metrics::new();
    m.inserts_total.store(5, Ordering::Relaxed);
    let j = m.to_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["inserts_total"], 5);
}

#[test]
fn test_shard_count_stored() {
    let m = Metrics::new();
    m.shard_count.store(16, Ordering::Relaxed);
    assert_eq!(m.shard_count.load(Ordering::Relaxed), 16);
}

#[test]
fn test_bytes_written_incremented() {
    let m = Metrics::new();
    m.bytes_written.fetch_add(4096, Ordering::Relaxed);
    assert_eq!(m.bytes_written.load(Ordering::Relaxed), 4096);
}

#[test]
fn test_active_connections() {
    let m = Metrics::new();
    m.active_connections.fetch_add(3, Ordering::Relaxed);
    m.active_connections.fetch_sub(1, Ordering::Relaxed);
    assert_eq!(m.active_connections.load(Ordering::Relaxed), 2);
}

#[test]
fn test_default_equals_new() {
    let m1 = Metrics::new();
    let m2 = Metrics::default();
    assert_eq!(
        m1.inserts_total.load(Ordering::Relaxed),
        m2.inserts_total.load(Ordering::Relaxed)
    );
}

#[test]
fn test_prometheus_contains_vv_prefix() {
    let m = Metrics::new();
    let text = m.to_prometheus_text();
    assert!(text.contains("vv_"));
}
