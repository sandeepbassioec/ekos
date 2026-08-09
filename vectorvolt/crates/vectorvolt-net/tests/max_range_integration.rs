//! Phase 100: MaxRange integration tests.

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
fn max_range_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.max_range(1, 100, 200).unwrap().is_none());
}

#[test]
fn max_range_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    assert_eq!(c.max_range(1, 99, 99).unwrap(), Some(99));
}

#[test]
fn max_range_returns_maximum() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.max_range(1, 10, 50).unwrap(), Some(50));
}

#[test]
fn max_range_all_same() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    assert_eq!(c.max_range(1, 5, 5).unwrap(), Some(5));
}

#[test]
fn max_range_partial_scan() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    // scan [10,20]: max of {10,20} = 20
    assert_eq!(c.max_range(1, 10, 20).unwrap(), Some(20));
}

#[test]
fn max_range_zero_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    assert_eq!(c.max_range(1, 0, 0).unwrap(), Some(0));
}
