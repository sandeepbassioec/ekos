//! Phase 78: PropagateMax integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "value".into()]
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
fn propagate_max_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.propagate_max(1, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn propagate_max_all_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    c.insert(3, vec![3, 5]).unwrap();
    // max is 5; all rows already have 5
    c.propagate_max(1, 5, 5).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 5);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 5);
}

#[test]
fn propagate_max_sets_all_to_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 10]).unwrap();
    c.insert(20, vec![20, 20]).unwrap();
    c.insert(30, vec![30, 30]).unwrap();
    // scan [10,30], max=30, all rows set to 30
    c.propagate_max(1, 10, 30).unwrap();
    assert_eq!(c.get(10).unwrap().unwrap()[1], 30);
    assert_eq!(c.get(20).unwrap().unwrap()[1], 30);
    assert_eq!(c.get(30).unwrap().unwrap()[1], 30);
}

#[test]
fn propagate_max_out_of_range_untouched() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    // scan [5,5], max=5; row with value=50 not in range
    c.propagate_max(1, 5, 5).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 5);
}

#[test]
fn propagate_max_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 42]).unwrap();
    c.propagate_max(1, 42, 42).unwrap();
    assert_eq!(c.get(7).unwrap().unwrap()[1], 42);
}

#[test]
fn propagate_max_returns_affected_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    // returns the max value found (30), not the row count
    let result = c.propagate_max(1, 10, 30).unwrap();
    assert_eq!(result, 30);
}
