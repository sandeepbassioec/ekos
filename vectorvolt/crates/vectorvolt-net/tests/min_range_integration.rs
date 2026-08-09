//! Phase 99: MinRange integration tests.

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
fn min_range_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.min_range(1, 100, 200).unwrap().is_none());
}

#[test]
fn min_range_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.min_range(1, 42, 42).unwrap(), Some(42));
}

#[test]
fn min_range_returns_minimum() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    assert_eq!(c.min_range(1, 5, 20).unwrap(), Some(5));
}

#[test]
fn min_range_all_same() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7]).unwrap();
    c.insert(2, vec![2, 7]).unwrap();
    assert_eq!(c.min_range(1, 7, 7).unwrap(), Some(7));
}

#[test]
fn min_range_partial_scan() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    // scan [10,20]: min of {10,20} = 10
    assert_eq!(c.min_range(1, 10, 20).unwrap(), Some(10));
}

#[test]
fn min_range_zero_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    assert_eq!(c.min_range(1, 0, 0).unwrap(), Some(0));
}
