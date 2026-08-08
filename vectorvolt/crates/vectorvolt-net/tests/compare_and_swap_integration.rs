//! Phase 23: CompareAndSwap integration tests.

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

// ── missing key ───────────────────────────────────────────────────────────────

#[test]
fn cas_missing_key_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.compare_and_swap(99, 0, 10, 20).unwrap();
    assert!(result.is_none());
}

// ── successful swap ───────────────────────────────────────────────────────────

#[test]
fn cas_matching_expected_swaps_and_returns_true() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();

    let swapped = c.compare_and_swap(1, 1, 42, 99).unwrap();
    assert_eq!(swapped, Some(true));
}

#[test]
fn cas_updates_stored_value_on_success() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 100]).unwrap();
    c.compare_and_swap(5, 1, 100, 200).unwrap();

    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row[1], 200);
}

// ── mismatch ──────────────────────────────────────────────────────────────────

#[test]
fn cas_mismatched_expected_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();

    let swapped = c.compare_and_swap(1, 1, 99, 50).unwrap();
    assert_eq!(swapped, Some(false));
}

#[test]
fn cas_mismatch_leaves_value_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 77]).unwrap();
    c.compare_and_swap(3, 1, 0, 999).unwrap();

    let row = c.get(3).unwrap().unwrap();
    assert_eq!(row[1], 77);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn cas_second_swap_on_same_value_fails() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 5]).unwrap();

    let first = c.compare_and_swap(7, 1, 5, 10).unwrap();
    assert_eq!(first, Some(true));

    // Value is now 10 — a second swap with expected=5 must fail.
    let second = c.compare_and_swap(7, 1, 5, 10).unwrap();
    assert_eq!(second, Some(false));
}

// ── does not affect sibling rows ──────────────────────────────────────────────

#[test]
fn cas_only_modifies_target_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    c.compare_and_swap(2, 1, 20, 200).unwrap();

    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 200);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 30);
}

// ── pk column ─────────────────────────────────────────────────────────────────

#[test]
fn cas_on_pk_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(4, vec![4, 0]).unwrap();

    let swapped = c.compare_and_swap(4, 0, 4, 999).unwrap();
    assert_eq!(swapped, Some(true));

    let row = c.get(4).unwrap().unwrap();
    assert_eq!(row[0], 999);
}
