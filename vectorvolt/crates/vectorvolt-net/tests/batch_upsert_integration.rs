//! Phase 14: BatchUpsert integration tests.

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
fn batch_upsert_empty_is_ok() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let affected = c.batch_upsert(vec![]).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn batch_upsert_inserts_new_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let affected = c
        .batch_upsert(vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])])
        .unwrap();
    assert_eq!(affected, 3);

    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 10]);
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 20]);
    assert_eq!(c.get(3).unwrap().unwrap(), vec![3, 30]);
}

#[test]
fn batch_upsert_replaces_existing_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    c.batch_upsert(vec![(1, vec![1, 100]), (2, vec![2, 200])])
        .unwrap();

    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 100]);
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 200]);
}

#[test]
fn batch_upsert_returns_submitted_count() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Mix of new and existing keys.
    c.insert(1, vec![1, 10]).unwrap();

    let rows = vec![(1, vec![1, 99]), (2, vec![2, 88]), (3, vec![3, 77])];
    let n = rows.len() as u64;
    let affected = c.batch_upsert(rows).unwrap();
    assert_eq!(affected, n);
}

// ── mixed new and existing ────────────────────────────────────────────────────

#[test]
fn batch_upsert_mixes_inserts_and_replaces() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 100]).unwrap(); // will be replaced
    // pk=20 does not exist — will be inserted

    c.batch_upsert(vec![(10, vec![10, 999]), (20, vec![20, 888])])
        .unwrap();

    assert_eq!(c.get(10).unwrap().unwrap(), vec![10, 999]);
    assert_eq!(c.get(20).unwrap().unwrap(), vec![20, 888]);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn batch_upsert_twice_leaves_last_values() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.batch_upsert(vec![(5, vec![5, 50]), (6, vec![6, 60])])
        .unwrap();
    c.batch_upsert(vec![(5, vec![5, 55]), (6, vec![6, 66])])
        .unwrap();

    assert_eq!(c.get(5).unwrap().unwrap(), vec![5, 55]);
    assert_eq!(c.get(6).unwrap().unwrap(), vec![6, 66]);
}

// ── single-row edge case ──────────────────────────────────────────────────────

#[test]
fn batch_upsert_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 420]).unwrap();
    let affected = c.batch_upsert(vec![(42, vec![42, 421])]).unwrap();
    assert_eq!(affected, 1);
    assert_eq!(c.get(42).unwrap().unwrap(), vec![42, 421]);
}

// ── visible via get-many ──────────────────────────────────────────────────────

#[test]
fn batch_upsert_visible_via_get_many() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.batch_upsert(vec![(100, vec![100, 1]), (200, vec![200, 2])])
        .unwrap();

    let result = c.get_many(vec![100, 200]).unwrap();
    let map: std::collections::HashMap<u64, Vec<u64>> = result.into_iter().collect();
    assert_eq!(map[&100], vec![100, 1]);
    assert_eq!(map[&200], vec![200, 2]);
}
