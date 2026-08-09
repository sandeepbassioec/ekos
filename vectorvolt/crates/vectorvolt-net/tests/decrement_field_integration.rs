//! Phase 103: DecrementField integration tests.

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
fn decrement_field_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.decrement_field(99, 1).unwrap().is_none());
}

#[test]
fn decrement_field_basic() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.decrement_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 9);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 9);
}

#[test]
fn decrement_field_saturates_at_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let v = c.decrement_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn decrement_field_cumulative() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.decrement_field(1, 1).unwrap();
    c.decrement_field(1, 1).unwrap();
    let v = c.decrement_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 2);
}

#[test]
fn decrement_field_from_one_to_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    let v = c.decrement_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn decrement_field_returns_new_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    let v = c.decrement_field(1, 1).unwrap().unwrap();
    assert_eq!(v, 99);
}
