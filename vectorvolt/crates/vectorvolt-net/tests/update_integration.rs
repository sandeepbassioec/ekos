//! Phase 11: Update integration tests.

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

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn update_existing_row_changes_values() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.update(1, vec![1, 999]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 999]);
}

#[test]
fn update_returns_ok() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    // update() returns () on success — if it returns Err the test panics.
    c.update(2, vec![2, 21]).unwrap();
}

#[test]
fn update_multiple_times_leaves_last_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 1]).unwrap();
    c.update(3, vec![3, 2]).unwrap();
    c.update(3, vec![3, 3]).unwrap();
    assert_eq!(c.get(3).unwrap().unwrap(), vec![3, 3]);
}

// ── error path ────────────────────────────────────────────────────────────────

#[test]
fn update_nonexistent_row_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert!(c.update(99, vec![99, 1]).is_err());
}

#[test]
fn update_after_delete_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(4, vec![4, 40]).unwrap();
    c.delete(4).unwrap();
    assert!(c.update(4, vec![4, 41]).is_err());
}

// ── interop ───────────────────────────────────────────────────────────────────

#[test]
fn get_after_update_returns_new_values() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    c.update(5, vec![5, 55]).unwrap();
    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row, vec![5, 55]);
}

#[test]
fn upsert_then_update_uses_updated_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.upsert(6, vec![6, 60]).unwrap();
    c.update(6, vec![6, 66]).unwrap();
    assert_eq!(c.get(6).unwrap().unwrap(), vec![6, 66]);
}

#[test]
fn update_row_visible_via_get_many() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 70]).unwrap();
    c.insert(8, vec![8, 80]).unwrap();
    c.update(7, vec![7, 77]).unwrap();

    let result = c.get_many(vec![7, 8]).unwrap();
    let map: std::collections::HashMap<u64, Vec<u64>> = result.into_iter().collect();
    assert_eq!(map[&7], vec![7, 77]);
    assert_eq!(map[&8], vec![8, 80]);
}
