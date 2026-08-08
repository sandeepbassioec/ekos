//! Phase 13: Count integration tests.

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
fn count_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.count(0, 0, u64::MAX).unwrap(), 0);
}

// ── full-table count ──────────────────────────────────────────────────────────

#[test]
fn count_all_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    assert_eq!(c.count(0, 0, u64::MAX).unwrap(), 3);
}

// ── pk range count ────────────────────────────────────────────────────────────

#[test]
fn count_pk_subrange() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=10 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }

    // PKs 3..=7 → 5 rows.
    assert_eq!(c.count(0, 3, 7).unwrap(), 5);
}

#[test]
fn count_exact_pk_match() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 420]).unwrap();

    assert_eq!(c.count(0, 42, 42).unwrap(), 1);
}

#[test]
fn count_range_with_no_matches() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    // Range [100, 200] — no pks there.
    assert_eq!(c.count(0, 100, 200).unwrap(), 0);
}

// ── non-pk column count ───────────────────────────────────────────────────────

#[test]
fn count_by_value_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // val column (col 1): values 10, 20, 30, 40, 50
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }

    // val in [20, 40] → 3 rows (val=20, 30, 40).
    assert_eq!(c.count(1, 20, 40).unwrap(), 3);
}

// ── count reflects mutations ──────────────────────────────────────────────────

#[test]
fn count_decreases_after_delete() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    let before = c.count(0, 0, u64::MAX).unwrap();
    c.delete(2).unwrap();
    // compact() flushes the tombstone and merges it with insert records so
    // the subsequent count scan sees the correct live-row set.
    c.compact().unwrap();
    let after = c.count(0, 0, u64::MAX).unwrap();

    assert_eq!(before, 3);
    assert_eq!(after, 2);
}

#[test]
fn count_zero_after_truncate() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 100]).unwrap();
    c.insert(20, vec![20, 200]).unwrap();
    c.truncate().unwrap();

    assert_eq!(c.count(0, 0, u64::MAX).unwrap(), 0);
}
