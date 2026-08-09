//! Phase 75: AbsDiff integration tests.

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
fn abs_diff_missing_pk1_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    assert!(c.abs_diff(99, 2, 1).unwrap().is_none());
}

#[test]
fn abs_diff_missing_pk2_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    assert!(c.abs_diff(1, 99, 1).unwrap().is_none());
}

#[test]
fn abs_diff_positive_difference() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 80]).unwrap();
    c.insert(2, vec![2, 30]).unwrap();
    assert_eq!(c.abs_diff(1, 2, 1).unwrap().unwrap(), 50);
}

#[test]
fn abs_diff_reverse_order_same_result() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 30]).unwrap();
    c.insert(2, vec![2, 80]).unwrap();
    assert_eq!(c.abs_diff(1, 2, 1).unwrap().unwrap(), 50);
}

#[test]
fn abs_diff_equal_values_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.insert(2, vec![2, 42]).unwrap();
    assert_eq!(c.abs_diff(1, 2, 1).unwrap().unwrap(), 0);
}

#[test]
fn abs_diff_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 40]).unwrap();
    c.abs_diff(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 40);
}
