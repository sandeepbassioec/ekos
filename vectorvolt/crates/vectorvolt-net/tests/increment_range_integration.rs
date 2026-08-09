//! Phase 65: IncrementRange integration tests.

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
fn increment_range_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.increment_range(1, 5, 0, 100).unwrap(), 0);
}

#[test]
fn increment_range_updates_all_in_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    let count = c.increment_range(1, 5, 0, 100).unwrap();
    assert_eq!(count, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 15);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 25);
}

#[test]
fn increment_range_respects_scan_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 50]).unwrap();
    c.increment_range(1, 10, 10, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5); // unchanged
    assert_eq!(c.get(2).unwrap().unwrap()[1], 60);
}

#[test]
fn increment_range_zero_delta_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.increment_range(1, 0, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn increment_range_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX - 1]).unwrap();
    c.increment_range(1, 5, 0, u64::MAX).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], u64::MAX);
}

#[test]
fn increment_range_returns_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let count = c.increment_range(1, 1, 0, 100).unwrap();
    assert_eq!(count, 5);
}

#[test]
fn increment_range_does_not_affect_out_of_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap();
    c.increment_range(1, 10, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 200);
}
