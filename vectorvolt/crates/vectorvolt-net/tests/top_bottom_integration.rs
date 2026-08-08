//! Phases 33 & 34: Top / Bottom integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "score".into()]
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

// ── Top ───────────────────────────────────────────────────────────────────────

#[test]
fn top_empty_table_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.top(1, 0, u64::MAX, 5).unwrap().is_empty());
}

#[test]
fn top_returns_rows_sorted_descending() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for (pk, score) in [(1u64, 30u64), (2, 10), (3, 50), (4, 20)] {
        c.insert(pk, vec![pk, score]).unwrap();
    }
    let rows = c.top(1, 0, u64::MAX, 0).unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0][1], 50);
    assert_eq!(rows[1][1], 30);
    assert_eq!(rows[2][1], 20);
    assert_eq!(rows[3][1], 10);
}

#[test]
fn top_n_limits_results() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    let rows = c.top(1, 0, u64::MAX, 3).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][1], 100);
    assert_eq!(rows[1][1], 90);
    assert_eq!(rows[2][1], 80);
}

#[test]
fn top_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 15]).unwrap(); // in range
    c.insert(3, vec![3, 25]).unwrap(); // in range
    c.insert(4, vec![4, 35]).unwrap();
    let rows = c.top(1, 10, 30, 0).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][1], 25);
    assert_eq!(rows[1][1], 15);
}

// ── Bottom ────────────────────────────────────────────────────────────────────

#[test]
fn bottom_empty_table_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.bottom(1, 0, u64::MAX, 5).unwrap().is_empty());
}

#[test]
fn bottom_returns_rows_sorted_ascending() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for (pk, score) in [(1u64, 30u64), (2, 10), (3, 50), (4, 20)] {
        c.insert(pk, vec![pk, score]).unwrap();
    }
    let rows = c.bottom(1, 0, u64::MAX, 0).unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0][1], 10);
    assert_eq!(rows[1][1], 20);
    assert_eq!(rows[2][1], 30);
    assert_eq!(rows[3][1], 50);
}

#[test]
fn bottom_n_limits_results() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    let rows = c.bottom(1, 0, u64::MAX, 3).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][1], 10);
    assert_eq!(rows[1][1], 20);
    assert_eq!(rows[2][1], 30);
}

#[test]
fn bottom_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 15]).unwrap(); // in range
    c.insert(3, vec![3, 25]).unwrap(); // in range
    c.insert(4, vec![4, 35]).unwrap();
    let rows = c.bottom(1, 10, 30, 0).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][1], 15);
    assert_eq!(rows[1][1], 25);
}
