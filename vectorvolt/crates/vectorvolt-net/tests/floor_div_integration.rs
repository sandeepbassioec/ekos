//! Phase 92: FloorDiv integration tests.

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
fn floor_div_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.floor_div(1, 2, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn floor_div_basic_division() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    // scan [10,10], divide by 3 → 10/3=3
    c.floor_div(1, 3, 10, 10).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 3);
}

#[test]
fn floor_div_divisor_zero_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    // divisor=0 should error
    let result = c.floor_div(1, 0, 10, 10);
    assert!(result.is_err());
}

#[test]
fn floor_div_exact_division() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 20]).unwrap();
    c.floor_div(1, 4, 20, 20).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5);
}

#[test]
fn floor_div_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let affected = c.floor_div(1, 10, 10, 30).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 1);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 2);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 3);
}

#[test]
fn floor_div_truncates_remainder() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7]).unwrap();
    c.floor_div(1, 2, 7, 7).unwrap();
    // 7 / 2 = 3 (floor)
    assert_eq!(c.get(1).unwrap().unwrap()[1], 3);
}
