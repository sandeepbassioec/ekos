//! Tests for Layer 312: ServerStats (AtomicU64 counters), Arc/clone patterns (VV-NET-312).

use std::sync::Arc;
use std::sync::atomic::Ordering;
use vectorvolt_net::server::ServerStats;

#[test]
fn stats_connections_accepted_default_zero() {
    let s = ServerStats::default();
    assert_eq!(s.connections_accepted.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_requests_handled_default_zero() {
    let s = ServerStats::default();
    assert_eq!(s.requests_handled.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_protocol_errors_default_zero() {
    let s = ServerStats::default();
    assert_eq!(s.protocol_errors.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_unauthenticated_rejections_zero() {
    let s = ServerStats::default();
    assert_eq!(s.unauthenticated_rejections.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_forbidden_rejections_zero() {
    let s = ServerStats::default();
    assert_eq!(s.forbidden_rejections.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_auth_failures_zero() {
    let s = ServerStats::default();
    assert_eq!(s.auth_failures.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_auth_successes_zero() {
    let s = ServerStats::default();
    assert_eq!(s.auth_successes.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_tls_handshake_failures_zero() {
    let s = ServerStats::default();
    assert_eq!(s.tls_handshake_failures.load(Ordering::Relaxed), 0);
}

#[test]
fn stats_connections_accepted_increment() {
    let s = ServerStats::default();
    s.connections_accepted.fetch_add(1, Ordering::Relaxed);
    assert_eq!(s.connections_accepted.load(Ordering::Relaxed), 1);
}

#[test]
fn stats_requests_handled_increment() {
    let s = ServerStats::default();
    s.requests_handled.fetch_add(5, Ordering::Relaxed);
    assert_eq!(s.requests_handled.load(Ordering::Relaxed), 5);
}

#[test]
fn stats_arc_shared() {
    let s = Arc::new(ServerStats::default());
    let s2 = Arc::clone(&s);
    s.connections_accepted.fetch_add(1, Ordering::Relaxed);
    assert_eq!(s2.connections_accepted.load(Ordering::Relaxed), 1);
}

#[test]
fn stats_debug_nonempty() {
    let s = ServerStats::default();
    assert!(!format!("{:?}", s).is_empty());
}

#[test]
fn stats_protocol_errors_increment() {
    let s = ServerStats::default();
    s.protocol_errors.fetch_add(3, Ordering::Relaxed);
    assert_eq!(s.protocol_errors.load(Ordering::Relaxed), 3);
}
