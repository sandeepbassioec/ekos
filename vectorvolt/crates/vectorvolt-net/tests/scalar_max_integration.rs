//! Phase 55: ScalarMax integration tests.

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

#[test]
fn scalar_max_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.scalar_max(1, 0, 100).unwrap().is_none());
}

#[test]
fn scalar_max_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.scalar_max(1, 0, 100).unwrap(), Some(42));
}

#[test]
fn scalar_max_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 80]).unwrap();
    c.insert(3, vec![3, 50]).unwrap();
    assert_eq!(c.scalar_max(1, 0, 100).unwrap(), Some(80));
}

#[test]
fn scalar_max_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 40]).unwrap();
    assert_eq!(c.scalar_max(1, 10, 100).unwrap(), Some(40));
}

#[test]
fn scalar_max_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.scalar_max(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn scalar_max_all_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, 42]).unwrap();
    }
    assert_eq!(c.scalar_max(1, 0, 100).unwrap(), Some(42));
}

#[test]
fn scalar_max_returns_max_u64() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    c.insert(2, vec![2, 1]).unwrap();
    assert_eq!(c.scalar_max(1, 0, u64::MAX).unwrap(), Some(u64::MAX));
}
