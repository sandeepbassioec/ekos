//! Tests for Layer 312: ConnectionPool stub (VV-CTL-312).

use vectorvolt_net::ConnectionPool;

#[test]
fn pool_new_stores_max() {
    let p = ConnectionPool::new(10);
    assert_eq!(p.max, 10);
}

#[test]
fn pool_size_returns_max() {
    let p = ConnectionPool::new(5);
    assert_eq!(p.size(), 5);
}

#[test]
fn pool_zero_capacity() {
    let p = ConnectionPool::new(0);
    assert_eq!(p.size(), 0);
}

#[test]
fn pool_large_capacity() {
    let p = ConnectionPool::new(1000);
    assert_eq!(p.size(), 1000);
}

#[test]
fn pool_debug_nonempty() {
    let p = ConnectionPool::new(3);
    assert!(!format!("{:?}", p).is_empty());
}

#[test]
fn pool_clone_equal() {
    let p = ConnectionPool::new(7);
    let p2 = p.clone();
    assert_eq!(p.max, p2.max);
}

#[test]
fn pool_max_field_accessible() {
    let p = ConnectionPool::new(42);
    let _ = p.max;
}

#[test]
fn pool_size_consistent() {
    let p = ConnectionPool::new(99);
    assert_eq!(p.max, p.size());
}

#[test]
fn pool_new_capacity_1() {
    let p = ConnectionPool::new(1);
    assert_eq!(p.size(), 1);
}

#[test]
fn pool_new_capacity_256() {
    let p = ConnectionPool::new(256);
    assert_eq!(p.size(), 256);
}

#[test]
fn pool_two_different_capacities() {
    let a = ConnectionPool::new(10);
    let b = ConnectionPool::new(20);
    assert_ne!(a.max, b.max);
}

#[test]
fn pool_clone_size_same() {
    let p = ConnectionPool::new(5);
    let p2 = p.clone();
    assert_eq!(p.size(), p2.size());
}

#[test]
fn pool_max_is_usize() {
    let p = ConnectionPool::new(usize::MAX);
    assert_eq!(p.max, usize::MAX);
}
