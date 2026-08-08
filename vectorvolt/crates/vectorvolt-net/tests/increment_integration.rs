//! Phase 22: Increment integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "counter".into()]
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

// ── missing key ───────────────────────────────────────────────────────────────

#[test]
fn increment_missing_key_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.increment(99, 0, 1).unwrap();
    assert!(result.is_none());
}

// ── basic increment ───────────────────────────────────────────────────────────

#[test]
fn increment_by_one_returns_new_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();

    let new_val = c.increment(1, 1, 1).unwrap();
    assert_eq!(new_val, Some(11));
}

#[test]
fn increment_updates_stored_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 100]).unwrap();
    c.increment(5, 1, 50).unwrap();

    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row[1], 150);
}

#[test]
fn increment_pk_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 0]).unwrap();

    let new_val = c.increment(3, 0, 7).unwrap();
    assert_eq!(new_val, Some(10));
}

// ── multiple increments ───────────────────────────────────────────────────────

#[test]
fn increment_accumulates_across_calls() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();

    for _ in 0..5 {
        c.increment(1, 1, 1).unwrap();
    }

    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 5);
}

// ── saturation ────────────────────────────────────────────────────────────────

#[test]
fn increment_saturates_at_u64_max() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX - 1]).unwrap();

    let new_val = c.increment(1, 1, 5).unwrap();
    assert_eq!(new_val, Some(u64::MAX));
}

// ── delta zero ────────────────────────────────────────────────────────────────

#[test]
fn increment_by_zero_returns_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 42]).unwrap();

    let new_val = c.increment(2, 1, 0).unwrap();
    assert_eq!(new_val, Some(42));
}

// ── does not affect sibling rows ──────────────────────────────────────────────

#[test]
fn increment_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    c.increment(2, 1, 100).unwrap();

    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 120);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 30);
}
