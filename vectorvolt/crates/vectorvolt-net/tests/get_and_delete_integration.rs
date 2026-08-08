//! Phase 21: GetAndDelete integration tests.

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
fn get_and_delete_missing_key_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.get_and_delete(99).unwrap();
    assert!(result.is_none());
}

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn get_and_delete_existing_row_returns_values() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();

    let result = c.get_and_delete(1).unwrap();
    assert_eq!(result, Some(vec![1, 42]));
}

#[test]
fn get_and_delete_removes_row_from_table() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 100]).unwrap();
    c.get_and_delete(5).unwrap();

    // Row must be gone after get-and-delete.
    let second = c.get_and_delete(5).unwrap();
    assert!(second.is_none());
}

#[test]
fn get_and_delete_only_removes_target_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    c.get_and_delete(2).unwrap();

    // Sibling rows must still exist.
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(3).unwrap().is_some());
    // Target row must be gone.
    assert!(c.get(2).unwrap().is_none());
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn get_and_delete_second_call_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 77]).unwrap();

    let first = c.get_and_delete(7).unwrap();
    assert!(first.is_some());

    let second = c.get_and_delete(7).unwrap();
    assert!(second.is_none());
}

// ── boundary keys ─────────────────────────────────────────────────────────────

#[test]
fn get_and_delete_pk_one() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 999]).unwrap();

    let result = c.get_and_delete(1).unwrap();
    assert_eq!(result, Some(vec![1, 999]));
}

#[test]
fn get_and_delete_large_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let large_pk = u64::MAX - 1;
    let mut c = Client::connect(addr).unwrap();
    c.insert(large_pk, vec![large_pk, 1]).unwrap();

    let result = c.get_and_delete(large_pk).unwrap();
    assert_eq!(result, Some(vec![large_pk, 1]));
}
