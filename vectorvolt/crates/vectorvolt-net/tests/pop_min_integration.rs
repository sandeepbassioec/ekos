//! Phase 27: PopMin integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "priority".into()]
}

fn start_server() -> (TempDir, Server, std::net::SocketAddr) {
    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open(schema(), tmp.path(), 1_048_576).unwrap());
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = ServerConfig::new(addr).with_auth(AuthConfig::disabled());
    let server = Server::bind(engine, cfg).unwrap();
    let local = server.local_addr().unwrap();
    (tmp, server, local)
}

fn background(server: Server) {
    std::thread::spawn(move || server.serve().unwrap());
}

// ── empty table ───────────────────────────────────────────────────────────────

#[test]
fn pop_min_empty_table_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert!(c.pop_min(0, 0, u64::MAX).unwrap().is_none());
}

// ── single row ────────────────────────────────────────────────────────────────

#[test]
fn pop_min_single_row_returns_and_removes_it() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();

    let row = c.pop_min(0, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(row[0], 1);
    assert!(c.get(1).unwrap().is_none());
}

// ── returns the minimum ───────────────────────────────────────────────────────

#[test]
fn pop_min_returns_row_with_minimum_col_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // pk=1 priority=30, pk=2 priority=10 (min), pk=3 priority=20
    c.insert(1, vec![1, 30]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();

    let row = c.pop_min(1, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(row[0], 2); // pk of minimum-priority row
    assert_eq!(row[1], 10);

    // Row 2 gone; rows 1 and 3 remain.
    assert!(c.get(2).unwrap().is_none());
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(3).unwrap().is_some());
}

// ── successive pops drain in order ───────────────────────────────────────────

#[test]
fn pop_min_successive_calls_drain_in_priority_order() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for (pk, pri) in [(1u64, 50u64), (2, 10), (3, 30), (4, 20)] {
        c.insert(pk, vec![pk, pri]).unwrap();
    }

    let first = c.pop_min(1, 0, u64::MAX).unwrap().unwrap();
    let second = c.pop_min(1, 0, u64::MAX).unwrap().unwrap();
    let third = c.pop_min(1, 0, u64::MAX).unwrap().unwrap();
    let fourth = c.pop_min(1, 0, u64::MAX).unwrap().unwrap();
    assert!(c.pop_min(1, 0, u64::MAX).unwrap().is_none());

    assert_eq!(first[1], 10);
    assert_eq!(second[1], 20);
    assert_eq!(third[1], 30);
    assert_eq!(fourth[1], 50);
}

// ── range filter ─────────────────────────────────────────────────────────────

#[test]
fn pop_min_respects_range_bounds() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // below range
    c.insert(2, vec![2, 15]).unwrap(); // in range — min
    c.insert(3, vec![3, 25]).unwrap(); // in range

    let row = c.pop_min(1, 10, 20).unwrap().unwrap();
    assert_eq!(row[1], 15);

    // Row with priority 5 (outside range) must still exist.
    assert!(c.get(1).unwrap().is_some());
}

// ── out-of-range returns not_found ────────────────────────────────────────────

#[test]
fn pop_min_out_of_range_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();

    assert!(c.pop_min(1, 100, 200).unwrap().is_none());
    // Original rows intact.
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(2).unwrap().is_some());
}

// ── idempotency after drain ───────────────────────────────────────────────────

#[test]
fn pop_min_after_full_drain_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();

    assert!(c.pop_min(1, 0, u64::MAX).unwrap().is_some());
    assert!(c.pop_min(1, 0, u64::MAX).unwrap().is_none());
}
