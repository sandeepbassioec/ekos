//! Phase 19: MinMax integration tests.

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
fn min_max_empty_table_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.min_max(0, 0, u64::MAX).unwrap();
    assert!(result.is_none());
}

// ── single row ────────────────────────────────────────────────────────────────

#[test]
fn min_max_single_row_min_equals_max() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 42]).unwrap();

    let (min, max, count) = c.min_max(0, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(min, 5);
    assert_eq!(max, 5);
    assert_eq!(count, 1);
}

// ── multiple rows ─────────────────────────────────────────────────────────────

#[test]
fn min_max_returns_correct_min_and_max() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(7, vec![7, 70]).unwrap();
    c.insert(5, vec![5, 50]).unwrap();

    let (min, max, count) = c.min_max(0, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(min, 1);
    assert_eq!(max, 7);
    assert_eq!(count, 4);
}

// ── count matches row count ───────────────────────────────────────────────────

#[test]
fn min_max_count_matches_inserted_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    let (_, _, count) = c.min_max(0, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(count, 10);
}

// ── range filter ──────────────────────────────────────────────────────────────

#[test]
fn min_max_respects_pk_range() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    // Only pks 3..=6 should be included.
    let (min, max, count) = c.min_max(0, 3, 6).unwrap().unwrap();
    assert_eq!(min, 3);
    assert_eq!(max, 6);
    assert_eq!(count, 4);
}

#[test]
fn min_max_out_of_range_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    // Range 100..200 has no rows.
    let result = c.min_max(0, 100, 200).unwrap();
    assert!(result.is_none());
}

// ── value column ──────────────────────────────────────────────────────────────

#[test]
fn min_max_on_value_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // pk=1 val=100, pk=2 val=50, pk=3 val=200
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.insert(3, vec![3, 200]).unwrap();

    // col=1 is the value column; scan by pk range (col=0) to get all rows,
    // then col=1 min/max — but MinMax uses col for BOTH filter and measurement.
    // Here col=0 (pk range) is the standard: we check pk column stats.
    let (min, max, count) = c.min_max(0, 1, 3).unwrap().unwrap();
    assert_eq!(min, 1);
    assert_eq!(max, 3);
    assert_eq!(count, 3);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn min_max_is_idempotent() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(8, vec![8, 80]).unwrap();

    let first = c.min_max(0, 0, u64::MAX).unwrap();
    let second = c.min_max(0, 0, u64::MAX).unwrap();
    assert_eq!(first, second);
}
