//! Phase 12: Truncate integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
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

// ── basic behaviour ───────────────────────────────────────────────────────────

#[test]
fn truncate_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let deleted = c.truncate().unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn truncate_returns_row_count() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    let deleted = c.truncate().unwrap();
    assert_eq!(deleted, 3);
}

#[test]
fn truncate_removes_all_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 100]).unwrap();
    c.insert(20, vec![20, 200]).unwrap();
    c.truncate().unwrap();

    assert!(c.get(10).unwrap().is_none());
    assert!(c.get(20).unwrap().is_none());
}

#[test]
fn truncate_then_insert_works() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.truncate().unwrap();

    // Table is empty — insert a fresh row.
    c.insert(1, vec![1, 99]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 99]);
}

#[test]
fn truncate_is_idempotent() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    c.truncate().unwrap();
    let deleted = c.truncate().unwrap();
    assert_eq!(deleted, 0);
}

// ── scan confirms empty after truncate ────────────────────────────────────────

#[test]
fn scan_returns_empty_after_truncate() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for i in 0u64..5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    c.truncate().unwrap();

    let rows = c.range_scan(0, 0, u64::MAX, 0).unwrap();
    assert!(rows.is_empty());
}

// ── truncate does not affect a second connection ──────────────────────────────

#[test]
fn truncate_on_one_connection_visible_on_another() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c1 = Client::connect(addr).unwrap();
    let mut c2 = Client::connect(addr).unwrap();

    c1.insert(100, vec![100, 1]).unwrap();
    c1.truncate().unwrap();

    // c2 should also see the empty table.
    assert!(c2.get(100).unwrap().is_none());
}
