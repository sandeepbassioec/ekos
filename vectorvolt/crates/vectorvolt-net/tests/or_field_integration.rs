//! Phase 108: OrField integration tests.

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
fn or_field_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.or_field(1, 0xFF, 100, 200).unwrap(), 0);
}

#[test]
fn or_field_sets_bits() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0b0000]).unwrap();
    c.or_field(1, 0b1010, 0, 0).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0b1010);
}

#[test]
fn or_field_zero_mask_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.or_field(1, 0, 42, 42).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn or_field_max_mask_sets_all() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.or_field(1, u64::MAX, 0, 0).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], u64::MAX);
}

#[test]
fn or_field_returns_affected_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    let affected = c.or_field(1, 0xFF, 0, 0).unwrap();
    assert_eq!(affected, 2);
}

#[test]
fn or_field_idempotent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0b1010]).unwrap();
    c.or_field(1, 0b1010, 0b1010, 0b1010).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0b1010);
}
