//! Phase 93: MulField integration tests.

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
fn mul_field_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.mul_field(1, 3, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn mul_field_basic_multiply() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.mul_field(1, 3, 5, 5).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 15);
}

#[test]
fn mul_field_factor_one_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.mul_field(1, 1, 42, 42).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn mul_field_factor_zero_zeroes_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.mul_field(1, 0, 99, 99).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn mul_field_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 2]).unwrap();
    c.insert(2, vec![2, 3]).unwrap();
    c.insert(3, vec![3, 4]).unwrap();
    let affected = c.mul_field(1, 10, 2, 4).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 20);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 30);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 40);
}

#[test]
fn mul_field_saturating_overflow() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    c.mul_field(1, 2, u64::MAX, u64::MAX).unwrap();
    // saturating_mul → u64::MAX
    assert_eq!(c.get(1).unwrap().unwrap()[1], u64::MAX);
}
