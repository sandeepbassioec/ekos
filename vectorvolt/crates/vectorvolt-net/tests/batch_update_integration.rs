//! Phase 15: BatchUpdate integration tests.

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
fn batch_update_empty_is_ok() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_update(vec![]).unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 0);
}

#[test]
fn batch_update_updates_existing_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    let (succeeded, skipped) = c
        .batch_update(vec![(1, vec![1, 100]), (2, vec![2, 200])])
        .unwrap();
    assert_eq!(succeeded, 2);
    assert_eq!(skipped, 0);

    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 100]);
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 200]);
}

#[test]
fn batch_update_skips_missing_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // No rows inserted — all should be skipped.
    let (succeeded, skipped) = c
        .batch_update(vec![(10, vec![10, 99]), (20, vec![20, 88])])
        .unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 2);

    assert!(c.get(10).unwrap().is_none());
    assert!(c.get(20).unwrap().is_none());
}

#[test]
fn batch_update_mixed_found_and_missing() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    // pk=6 does not exist

    let (succeeded, skipped) = c
        .batch_update(vec![(5, vec![5, 555]), (6, vec![6, 666])])
        .unwrap();
    assert_eq!(succeeded, 1);
    assert_eq!(skipped, 1);

    assert_eq!(c.get(5).unwrap().unwrap(), vec![5, 555]);
    assert!(c.get(6).unwrap().is_none());
}

#[test]
fn batch_update_does_not_insert_new_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_update(vec![(99, vec![99, 0])]).unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 1);
    assert!(c.get(99).unwrap().is_none());
}

// ── return counts ─────────────────────────────────────────────────────────────

#[test]
fn batch_update_returns_correct_counts() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=5 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    // Update 3 existing + 2 missing
    let rows = vec![
        (1, vec![1, 11]),
        (2, vec![2, 22]),
        (3, vec![3, 33]),
        (100, vec![100, 0]),
        (200, vec![200, 0]),
    ];
    let (succeeded, skipped) = c.batch_update(rows).unwrap();
    assert_eq!(succeeded, 3);
    assert_eq!(skipped, 2);
}

// ── interop ───────────────────────────────────────────────────────────────────

#[test]
fn batch_update_visible_via_get_many() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 1]).unwrap();
    c.insert(20, vec![20, 2]).unwrap();

    c.batch_update(vec![(10, vec![10, 100]), (20, vec![20, 200])])
        .unwrap();

    let result = c.get_many(vec![10, 20]).unwrap();
    let map: std::collections::HashMap<u64, Vec<u64>> = result.into_iter().collect();
    assert_eq!(map[&10], vec![10, 100]);
    assert_eq!(map[&20], vec![20, 200]);
}

#[test]
fn batch_update_after_batch_insert() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.batch_insert(vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])])
        .unwrap();

    let (succeeded, skipped) = c
        .batch_update(vec![(1, vec![1, 11]), (2, vec![2, 22]), (99, vec![99, 0])])
        .unwrap();
    assert_eq!(succeeded, 2);
    assert_eq!(skipped, 1);

    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 11]);
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 22]);
    assert_eq!(c.get(3).unwrap().unwrap(), vec![3, 30]); // untouched
}
