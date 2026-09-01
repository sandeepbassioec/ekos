//! Tests for Layer 311: Shutdown / ServerStats handle (VV-CTL-311).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use vectorvolt_net::server::ServerStats;

#[test]
fn server_stats_default_connections_zero() {
    let stats = ServerStats::default();
    assert_eq!(stats.connections_accepted.load(Ordering::SeqCst), 0);
}

#[test]
fn server_stats_default_requests_zero() {
    let stats = ServerStats::default();
    assert_eq!(stats.requests_handled.load(Ordering::SeqCst), 0);
}

#[test]
fn server_stats_default_errors_zero() {
    let stats = ServerStats::default();
    assert_eq!(stats.protocol_errors.load(Ordering::SeqCst), 0);
}

#[test]
fn server_stats_increment_connections() {
    let stats = ServerStats::default();
    stats.connections_accepted.fetch_add(1, Ordering::SeqCst);
    assert_eq!(stats.connections_accepted.load(Ordering::SeqCst), 1);
}

#[test]
fn server_stats_increment_requests() {
    let stats = ServerStats::default();
    stats.requests_handled.fetch_add(5, Ordering::SeqCst);
    assert_eq!(stats.requests_handled.load(Ordering::SeqCst), 5);
}

#[test]
fn atomic_bool_shutdown_signal() {
    let flag = Arc::new(AtomicBool::new(false));
    let flag2 = Arc::clone(&flag);
    flag2.store(true, Ordering::SeqCst);
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn atomic_bool_initial_false() {
    let flag = AtomicBool::new(false);
    assert!(!flag.load(Ordering::SeqCst));
}

#[test]
fn atomic_bool_set_true() {
    let flag = AtomicBool::new(false);
    flag.store(true, Ordering::SeqCst);
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn arc_atomic_clone() {
    let a = Arc::new(AtomicBool::new(false));
    let b = Arc::clone(&a);
    a.store(true, Ordering::SeqCst);
    assert!(b.load(Ordering::SeqCst));
}

#[test]
fn server_stats_debug_nonempty() {
    let stats = ServerStats::default();
    assert!(!format!("{:?}", stats).is_empty());
}

#[test]
fn server_stats_unauthenticated_rejections_zero() {
    let stats = ServerStats::default();
    assert_eq!(stats.unauthenticated_rejections.load(Ordering::SeqCst), 0);
}

#[test]
fn atomic_u64_initial_zero() {
    let a = AtomicU64::new(0);
    assert_eq!(a.load(Ordering::SeqCst), 0);
}

#[test]
fn atomic_u64_store_and_load() {
    let a = AtomicU64::new(0);
    a.store(42, Ordering::SeqCst);
    assert_eq!(a.load(Ordering::SeqCst), 42);
}
