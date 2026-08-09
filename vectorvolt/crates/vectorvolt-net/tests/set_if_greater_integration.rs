//! Phase 73: SetIfGreater integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "floor".into()]
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
fn set_if_greater_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.set_if_greater(1, 50, 0, 100).unwrap(), 0);
}

#[test]
fn set_if_greater_raises_low_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    let count = c.set_if_greater(1, 50, 0, 100).unwrap();
    assert_eq!(count, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 50);
}

#[test]
fn set_if_greater_skips_already_high() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 80]).unwrap();
    let count = c.set_if_greater(1, 50, 0, 100).unwrap();
    assert_eq!(count, 0);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 80);
}

#[test]
fn set_if_greater_skips_equal() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    let count = c.set_if_greater(1, 50, 0, 100).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn set_if_greater_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 20]).unwrap();
    c.set_if_greater(1, 50, 10, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5); // unchanged
    assert_eq!(c.get(2).unwrap().unwrap()[1], 50);
}

#[test]
fn set_if_greater_partial_update() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 90]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let count = c.set_if_greater(1, 50, 0, 100).unwrap();
    assert_eq!(count, 2); // pk=1 and pk=3 updated, pk=2 skipped
    assert_eq!(c.get(2).unwrap().unwrap()[1], 90);
}
