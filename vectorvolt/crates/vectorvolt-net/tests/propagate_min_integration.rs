//! Phase 79: PropagateMin integration tests.

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
fn propagate_min_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.propagate_min(1, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn propagate_min_sets_all_to_min() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 10]).unwrap();
    c.insert(20, vec![20, 20]).unwrap();
    c.insert(30, vec![30, 30]).unwrap();
    // scan [10,30], min=10, all rows set to 10
    c.propagate_min(1, 10, 30).unwrap();
    assert_eq!(c.get(10).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(20).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(30).unwrap().unwrap()[1], 10);
}

#[test]
fn propagate_min_out_of_range_untouched() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    // scan [50,50], min=50; row with value=5 not in range
    c.propagate_min(1, 50, 50).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 50);
}

#[test]
fn propagate_min_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 99]).unwrap();
    c.propagate_min(1, 99, 99).unwrap();
    assert_eq!(c.get(7).unwrap().unwrap()[1], 99);
}

#[test]
fn propagate_min_returns_min_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let result = c.propagate_min(1, 10, 30).unwrap();
    assert_eq!(result, 10);
}

#[test]
fn propagate_min_all_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7]).unwrap();
    c.insert(2, vec![2, 7]).unwrap();
    c.propagate_min(1, 7, 7).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 7);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 7);
}
