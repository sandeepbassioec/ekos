//! Tests for Layer 195: vectorvolt-net ConnectionPool (VV-NET-195).

use vectorvolt_net::ConnectionPool;

#[test]
fn new_stores_max() {
    assert_eq!(ConnectionPool::new(10).max, 10);
}
#[test]
fn size_equals_max() {
    assert_eq!(ConnectionPool::new(5).size(), 5);
}
#[test]
fn zero_max() {
    assert_eq!(ConnectionPool::new(0).size(), 0);
}
#[test]
fn large_max() {
    assert_eq!(ConnectionPool::new(10000).size(), 10000);
}
#[test]
fn clone_preserves_max() {
    let p = ConnectionPool::new(7);
    assert_eq!(p.clone().max, 7);
}
#[test]
fn debug_contains_max() {
    let p = ConnectionPool::new(42);
    let s = format!("{:?}", p);
    assert!(s.contains("42"));
}
#[test]
fn size_consistent() {
    let p = ConnectionPool::new(3);
    assert_eq!(p.size(), p.max);
}
#[test]
fn multiple_pools() {
    let a = ConnectionPool::new(1);
    let b = ConnectionPool::new(2);
    assert_ne!(a.size(), b.size());
}
#[test]
fn max_1() {
    assert_eq!(ConnectionPool::new(1).size(), 1);
}
#[test]
fn max_usize_max() {
    let p = ConnectionPool::new(usize::MAX);
    assert_eq!(p.size(), usize::MAX);
}
#[test]
fn pool_max_field() {
    let p = ConnectionPool::new(99);
    assert_eq!(p.max, 99);
}
#[test]
fn clone_is_independent() {
    let a = ConnectionPool::new(8);
    let b = a.clone();
    assert_eq!(a.size(), b.size());
}
#[test]
fn new_returns_correct_type() {
    let _: ConnectionPool = ConnectionPool::new(4);
}
