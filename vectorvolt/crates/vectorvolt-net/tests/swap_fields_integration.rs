//! Phase 76: SwapFields integration tests.

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
fn swap_fields_missing_pk1_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    assert!(c.swap_fields(99, 2, 1).unwrap().is_none());
}

#[test]
fn swap_fields_missing_pk2_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    assert!(c.swap_fields(1, 99, 1).unwrap().is_none());
}

#[test]
fn swap_fields_exchanges_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    let result = c.swap_fields(1, 2, 1).unwrap();
    assert!(result.is_some());
    assert_eq!(c.get(1).unwrap().unwrap()[1], 200);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 100);
}

#[test]
fn swap_fields_idempotent_twice() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 90]).unwrap();
    c.swap_fields(1, 2, 1).unwrap();
    c.swap_fields(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 90);
}

#[test]
fn swap_fields_zero_and_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 77]).unwrap();
    c.swap_fields(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 0);
}

#[test]
fn swap_fields_same_value_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.insert(2, vec![2, 42]).unwrap();
    c.swap_fields(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 42);
}
