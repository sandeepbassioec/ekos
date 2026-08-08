//! Phase 10: GetMany integration tests.

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

// ── basic retrieval ───────────────────────────────────────────────────────────

#[test]
fn get_many_empty_pks_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.get_many(vec![]).unwrap();
    assert!(result.is_empty());
}

#[test]
fn get_many_all_missing_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let result = c.get_many(vec![1, 2, 3]).unwrap();
    assert!(result.is_empty());
}

#[test]
fn get_many_returns_found_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 100]).unwrap();
    c.insert(20, vec![20, 200]).unwrap();
    c.insert(30, vec![30, 300]).unwrap();

    let result = c.get_many(vec![10, 20, 30]).unwrap();
    assert_eq!(result.len(), 3);
    // Convert to map for order-independent assertion.
    let map: std::collections::HashMap<u64, Vec<u64>> = result.into_iter().collect();
    assert_eq!(map[&10], vec![10, 100]);
    assert_eq!(map[&20], vec![20, 200]);
    assert_eq!(map[&30], vec![30, 300]);
}

#[test]
fn get_many_skips_missing_keys() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    c.insert(7, vec![7, 70]).unwrap();

    // Request 5 keys, only 2 exist.
    let result = c.get_many(vec![5, 6, 7, 8, 9]).unwrap();
    assert_eq!(result.len(), 2);
    let map: std::collections::HashMap<u64, Vec<u64>> = result.into_iter().collect();
    assert_eq!(map[&5], vec![5, 50]);
    assert_eq!(map[&7], vec![7, 70]);
    assert!(!map.contains_key(&6));
    assert!(!map.contains_key(&8));
}

#[test]
fn get_many_single_key() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 420]).unwrap();

    let result = c.get_many(vec![42]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (42, vec![42, 420]));
}

// ── interaction with writes ───────────────────────────────────────────────────

#[test]
fn get_many_reflects_upsert() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.upsert(1, vec![1, 99]).unwrap();

    let result = c.get_many(vec![1]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].1, vec![1, 99]);
}

#[test]
fn get_many_does_not_return_deleted_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(11, vec![11, 110]).unwrap();
    c.insert(12, vec![12, 120]).unwrap();
    c.delete(11).unwrap();

    let result = c.get_many(vec![11, 12]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 12);
}
