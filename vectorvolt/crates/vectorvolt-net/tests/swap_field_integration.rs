//! Phase 38: SwapField integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "score".into()]
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
fn swap_field_missing_pk1_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 99]).unwrap();
    assert!(!c.swap_field(1, 2, 1).unwrap());
}

#[test]
fn swap_field_missing_pk2_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    assert!(!c.swap_field(1, 99, 1).unwrap());
}

#[test]
fn swap_field_exchanges_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    assert!(c.swap_field(1, 2, 1).unwrap());
    assert_eq!(c.get(1).unwrap().unwrap()[1], 20);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 10);
}

#[test]
fn swap_field_equal_values_is_noop() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.insert(2, vec![2, 42]).unwrap();
    assert!(c.swap_field(1, 2, 1).unwrap());
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 42);
}

#[test]
fn swap_field_does_not_change_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 7]).unwrap();
    c.insert(5, vec![5, 9]).unwrap();
    c.swap_field(3, 5, 1).unwrap();
    assert_eq!(c.get(3).unwrap().unwrap()[0], 3);
    assert_eq!(c.get(5).unwrap().unwrap()[0], 5);
}

#[test]
fn swap_field_is_reversible() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    c.swap_field(1, 2, 1).unwrap();
    c.swap_field(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 200);
}

#[test]
fn swap_field_both_missing_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(!c.swap_field(1, 2, 1).unwrap());
}
