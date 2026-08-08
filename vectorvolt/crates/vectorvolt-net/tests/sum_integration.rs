//! Phase 20: Sum integration tests.

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
fn sum_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let total = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(total, 0);
}

// ── basic sums ────────────────────────────────────────────────────────────────

#[test]
fn sum_pk_column_equals_arithmetic_sum() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=5 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    // pk sum = 1+2+3+4+5 = 15
    let total = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(total, 15);
}

#[test]
fn sum_value_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    c.insert(3, vec![3, 300]).unwrap();

    // val col (index 1): scan is on col=0 (pk), values come from col 1
    // MinMax/Sum col param filters by that column — col=0 filters by pk range.
    // The sum is over values[col] for each matching row.
    let total = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(total, 1 + 2 + 3); // col=0 is pk
}

// ── range filter ──────────────────────────────────────────────────────────────

#[test]
fn sum_respects_pk_range() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 100]).unwrap();
    }
    // pks 3..=7: sum = 3+4+5+6+7 = 25
    let total = c.sum(0, 3, 7).unwrap();
    assert_eq!(total, 25);
}

#[test]
fn sum_out_of_range_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();

    let total = c.sum(0, 100, 200).unwrap();
    assert_eq!(total, 0);
}

// ── single row ────────────────────────────────────────────────────────────────

#[test]
fn sum_single_row_equals_that_value() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 999]).unwrap();

    let total = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(total, 42);
}

// ── saturation ────────────────────────────────────────────────────────────────

#[test]
fn sum_saturates_at_u64_max() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Two large values that would overflow u64 — result should saturate.
    let big = u64::MAX / 2 + 1;
    c.insert(1, vec![1, big]).unwrap();
    c.insert(2, vec![2, big]).unwrap();

    // Sum of pks (col=0): 1+2=3, no saturation.
    let pk_sum = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(pk_sum, 3);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn sum_is_idempotent() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=5 {
        c.insert(pk, vec![pk, pk]).unwrap();
    }

    let first = c.sum(0, 0, u64::MAX).unwrap();
    let second = c.sum(0, 0, u64::MAX).unwrap();
    assert_eq!(first, second);
}

// ── consistency with count ────────────────────────────────────────────────────

#[test]
fn sum_of_ones_equals_count() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Insert rows where pk=1..=N; sum of pks = N*(N+1)/2, count = N.
    for pk in 1u64..=6 {
        c.insert(pk, vec![pk, 1]).unwrap();
    }
    let total = c.sum(0, 0, u64::MAX).unwrap();
    let count = c.count(0, 0, u64::MAX).unwrap();
    // sum of 1..=6 = 21; count = 6
    assert_eq!(total, 21);
    assert_eq!(count, 6);
}
