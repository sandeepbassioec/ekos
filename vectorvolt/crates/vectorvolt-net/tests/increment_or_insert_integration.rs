//! Phase 57: IncrementOrInsert integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "counter".into()]
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

#[test]
fn increment_or_insert_inserts_when_absent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let new_val = c.increment_or_insert(1, 1, 5, vec![1, 0]).unwrap();
    assert_eq!(new_val, 0);
    assert!(c.get(1).unwrap().is_some());
}

#[test]
fn increment_or_insert_increments_when_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let new_val = c.increment_or_insert(1, 1, 5, vec![1, 0]).unwrap();
    assert_eq!(new_val, 15);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 15);
}

#[test]
fn increment_or_insert_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    let new_val = c.increment_or_insert(1, 1, 1, vec![1, 0]).unwrap();
    assert_eq!(new_val, u64::MAX);
}

#[test]
fn increment_or_insert_zero_delta_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let new_val = c.increment_or_insert(1, 1, 0, vec![1, 0]).unwrap();
    assert_eq!(new_val, 42);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn increment_or_insert_inserts_with_correct_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.increment_or_insert(5, 1, 10, vec![5, 99]).unwrap();
    let row = c.get(5).unwrap().unwrap();
    assert_eq!(row[0], 5);
    assert_eq!(row[1], 99);
}

#[test]
fn increment_or_insert_multiple_increments() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.increment_or_insert(1, 1, 1, vec![1, 0]).unwrap();
    c.increment_or_insert(1, 1, 1, vec![1, 0]).unwrap();
    let new_val = c.increment_or_insert(1, 1, 1, vec![1, 0]).unwrap();
    assert_eq!(new_val, 2);
}

#[test]
fn increment_or_insert_independent_keys() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.increment_or_insert(2, 1, 10, vec![2, 0]).unwrap();
    // pk=1 unchanged
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5);
}
