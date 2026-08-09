//! Phase 105: SetMax integration tests.

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
fn set_max_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.set_max(99, 1, 5).unwrap().is_none());
}

#[test]
fn set_max_value_larger_updates() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.set_max(1, 1, 50).unwrap().unwrap();
    assert_eq!(v, 50);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
}

#[test]
fn set_max_value_smaller_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.set_max(1, 1, 3).unwrap().unwrap();
    assert_eq!(v, 10);
}

#[test]
fn set_max_equal_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.set_max(1, 1, 10).unwrap().unwrap();
    assert_eq!(v, 10);
}

#[test]
fn set_max_from_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let v = c.set_max(1, 1, 42).unwrap().unwrap();
    assert_eq!(v, 42);
}

#[test]
fn set_max_already_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    let v = c.set_max(1, 1, 0).unwrap().unwrap();
    assert_eq!(v, u64::MAX);
}
