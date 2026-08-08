//! Phase 26: DeleteRange integration tests.

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

// ── empty table ───────────────────────────────────────────────────────────────

#[test]
fn delete_range_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let count = c.delete_range(0, 0, u64::MAX).unwrap();
    assert_eq!(count, 0);
}

// ── delete all ────────────────────────────────────────────────────────────────

#[test]
fn delete_range_all_rows_returns_count() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=5 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    let count = c.delete_range(0, 0, u64::MAX).unwrap();
    assert_eq!(count, 5);
}

#[test]
fn delete_range_all_rows_leaves_table_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=4 {
        c.insert(pk, vec![pk, 0]).unwrap();
    }
    c.delete_range(0, 0, u64::MAX).unwrap();

    for pk in 1u64..=4 {
        assert!(c.get(pk).unwrap().is_none());
    }
}

// ── partial range ─────────────────────────────────────────────────────────────

#[test]
fn delete_range_partial_deletes_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=6 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    // Delete pks 3..=5
    let count = c.delete_range(0, 3, 5).unwrap();
    assert_eq!(count, 3);

    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(2).unwrap().is_some());
    assert!(c.get(3).unwrap().is_none());
    assert!(c.get(4).unwrap().is_none());
    assert!(c.get(5).unwrap().is_none());
    assert!(c.get(6).unwrap().is_some());
}

// ── out-of-range ──────────────────────────────────────────────────────────────

#[test]
fn delete_range_no_match_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    let count = c.delete_range(0, 100, 200).unwrap();
    assert_eq!(count, 0);

    // Rows still exist.
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(2).unwrap().is_some());
}

// ── single row ────────────────────────────────────────────────────────────────

#[test]
fn delete_range_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 0]).unwrap();

    let count = c.delete_range(0, 7, 7).unwrap();
    assert_eq!(count, 1);
    assert!(c.get(7).unwrap().is_none());
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn delete_range_second_call_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=3 {
        c.insert(pk, vec![pk, 0]).unwrap();
    }

    let first = c.delete_range(0, 0, u64::MAX).unwrap();
    let second = c.delete_range(0, 0, u64::MAX).unwrap();
    assert_eq!(first, 3);
    assert_eq!(second, 0);
}
