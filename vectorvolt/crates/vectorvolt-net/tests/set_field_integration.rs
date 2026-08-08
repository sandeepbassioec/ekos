//! Phase 24: SetField integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "a".into(), "b".into()]
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
fn set_field_missing_key_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let updated = c.set_field(99, 1, 42).unwrap();
    assert!(!updated);
}

// ── basic update ──────────────────────────────────────────────────────────────

#[test]
fn set_field_updates_target_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();

    let updated = c.set_field(1, 1, 99).unwrap();
    assert!(updated);

    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 99);
}

#[test]
fn set_field_leaves_other_columns_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 10, 20]).unwrap();

    c.set_field(2, 1, 55).unwrap();

    let row = c.get(2).unwrap().unwrap();
    assert_eq!(row[0], 2); // pk unchanged
    assert_eq!(row[1], 55); // col 1 updated
    assert_eq!(row[2], 20); // col 2 unchanged
}

#[test]
fn set_field_second_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 10, 20]).unwrap();

    c.set_field(3, 2, 777).unwrap();

    let row = c.get(3).unwrap().unwrap();
    assert_eq!(row[1], 10); // col 1 unchanged
    assert_eq!(row[2], 777); // col 2 updated
}

// ── pk column ─────────────────────────────────────────────────────────────────

#[test]
fn set_field_pk_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(4, vec![4, 0, 0]).unwrap();

    let updated = c.set_field(4, 0, 999).unwrap();
    assert!(updated);

    let row = c.get(4).unwrap().unwrap();
    assert_eq!(row[0], 999);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn set_field_twice_takes_last_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 1, 2]).unwrap();

    c.set_field(5, 1, 100).unwrap();
    c.set_field(5, 1, 200).unwrap();

    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row[1], 200);
}

// ── does not affect sibling rows ──────────────────────────────────────────────

#[test]
fn set_field_only_modifies_target_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 0]).unwrap();
    c.insert(2, vec![2, 20, 0]).unwrap();
    c.insert(3, vec![3, 30, 0]).unwrap();

    c.set_field(2, 1, 999).unwrap();

    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 999);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 30);
}
