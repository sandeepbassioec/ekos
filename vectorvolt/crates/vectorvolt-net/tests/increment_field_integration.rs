//! Phase 102: IncrementField integration tests.

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
fn increment_field_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.increment_field(99, 1).unwrap().is_none());
}

#[test]
fn increment_field_basic() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.increment_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 11);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 11);
}

#[test]
fn increment_field_cumulative() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.increment_field(1, 1).unwrap();
    c.increment_field(1, 1).unwrap();
    let v = c.increment_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 3);
}

#[test]
fn increment_field_from_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let v = c.increment_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 1);
}

#[test]
fn increment_field_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    let v = c.increment_field(1, 1).unwrap().unwrap();
    assert_eq!(v, u64::MAX);
}

#[test]
fn increment_field_returns_new_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    let v = c.increment_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 100);
}
