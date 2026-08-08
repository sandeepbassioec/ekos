//! Phase 25: InsertIfAbsent integration tests.

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

// ── new key ───────────────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_new_key_returns_true() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let inserted = c.insert_if_absent(1, vec![1, 42]).unwrap();
    assert!(inserted);
}

#[test]
fn insert_if_absent_new_key_stores_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert_if_absent(5, vec![5, 99]).unwrap();

    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row[1], 99);
}

// ── existing key ──────────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_existing_key_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 10]).unwrap();

    let inserted = c.insert_if_absent(2, vec![2, 999]).unwrap();
    assert!(!inserted);
}

#[test]
fn insert_if_absent_existing_key_leaves_original_intact() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 77]).unwrap();
    c.insert_if_absent(3, vec![3, 999]).unwrap();

    let row = c.get(3).unwrap().unwrap();
    assert_eq!(row[1], 77); // original value preserved
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_second_call_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let first = c.insert_if_absent(7, vec![7, 1]).unwrap();
    let second = c.insert_if_absent(7, vec![7, 2]).unwrap();
    assert!(first);
    assert!(!second);
}

// ── multiple rows ─────────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    // Insert pk=2 (absent), attempt pk=1 (present).
    c.insert_if_absent(2, vec![2, 20]).unwrap();
    c.insert_if_absent(1, vec![1, 999]).unwrap();

    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 20);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 30);
}

// ── boundary ──────────────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_large_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let large_pk = u64::MAX - 1;
    let mut c = Client::connect(addr).unwrap();
    let inserted = c.insert_if_absent(large_pk, vec![large_pk, 0]).unwrap();
    assert!(inserted);

    let row = c.get(large_pk).unwrap().unwrap();
    assert_eq!(row[0], large_pk);
}
