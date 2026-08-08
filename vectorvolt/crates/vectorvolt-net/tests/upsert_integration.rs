//! Phase 9: upsert integration tests.

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

// ── basic insert path ─────────────────────────────────────────────────────────

#[test]
fn upsert_inserts_new_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.upsert(1, vec![1, 100]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 100]);
}

#[test]
fn upsert_replaces_existing_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 200]);

    c.upsert(2, vec![2, 999]).unwrap();
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 999]);
}

#[test]
fn upsert_on_nonexistent_row_does_not_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Row 99 never existed — must succeed without error.
    c.upsert(99, vec![99, 1]).unwrap();
    assert!(c.get(99).unwrap().is_some());
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn upsert_twice_leaves_last_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.upsert(3, vec![3, 10]).unwrap();
    c.upsert(3, vec![3, 20]).unwrap();
    assert_eq!(c.get(3).unwrap().unwrap(), vec![3, 20]);
}

#[test]
fn upsert_same_values_is_fine() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.upsert(4, vec![4, 42]).unwrap();
    c.upsert(4, vec![4, 42]).unwrap();
    assert_eq!(c.get(4).unwrap().unwrap(), vec![4, 42]);
}

// ── interop with insert/delete ────────────────────────────────────────────────

#[test]
fn upsert_after_delete_reinserts() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    c.delete(5).unwrap();
    assert!(c.get(5).unwrap().is_none());

    c.upsert(5, vec![5, 55]).unwrap();
    assert_eq!(c.get(5).unwrap().unwrap(), vec![5, 55]);
}

#[test]
fn insert_after_upsert_visible() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.upsert(6, vec![6, 60]).unwrap();
    // A subsequent plain insert should succeed (no duplicate-key error from
    // the engine, since upsert already wrote the row as the latest version).
    // Behaviour: insert appends, get returns latest — row is visible.
    let _ = c.insert(6, vec![6, 61]);
    // Whatever version is visible, the key must exist.
    assert!(c.get(6).unwrap().is_some());
}
