//! Tests for Layer 197: vectorvolt-server metrics (VV-SRV-197).

use vectorvolt_server::ServerMetrics;

#[test]
fn new_requests_zero() {
    assert_eq!(ServerMetrics::new().requests, 0);
}
#[test]
fn new_errors_zero() {
    assert_eq!(ServerMetrics::new().errors, 0);
}
#[test]
fn record_request_increments() {
    let mut m = ServerMetrics::new();
    m.record_request();
    assert_eq!(m.requests, 1);
}
#[test]
fn record_error_increments() {
    let mut m = ServerMetrics::new();
    m.record_error();
    assert_eq!(m.errors, 1);
}
#[test]
fn multiple_requests() {
    let mut m = ServerMetrics::new();
    for _ in 0..5 {
        m.record_request();
    }
    assert_eq!(m.requests, 5);
}
#[test]
fn multiple_errors() {
    let mut m = ServerMetrics::new();
    for _ in 0..3 {
        m.record_error();
    }
    assert_eq!(m.errors, 3);
}
#[test]
fn requests_and_errors_independent() {
    let mut m = ServerMetrics::new();
    m.record_request();
    m.record_error();
    assert_eq!(m.requests, 1);
    assert_eq!(m.errors, 1);
}
#[test]
fn default_requests_zero() {
    assert_eq!(ServerMetrics::default().requests, 0);
}
#[test]
fn default_errors_zero() {
    assert_eq!(ServerMetrics::default().errors, 0);
}
#[test]
fn clone_preserves_requests() {
    let mut m = ServerMetrics::new();
    m.record_request();
    assert_eq!(m.clone().requests, 1);
}
#[test]
fn clone_preserves_errors() {
    let mut m = ServerMetrics::new();
    m.record_error();
    assert_eq!(m.clone().errors, 1);
}
#[test]
fn debug_contains_requests() {
    let s = format!("{:?}", ServerMetrics::new());
    assert!(s.contains("requests"));
}
#[test]
fn many_increments() {
    let mut m = ServerMetrics::new();
    for _ in 0..100 {
        m.record_request();
        m.record_error();
    }
    assert_eq!(m.requests, 100);
    assert_eq!(m.errors, 100);
}
