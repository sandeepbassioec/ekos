//! Phase 77: CopyRange integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "col_a".into(), "col_b".into()]
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
fn copy_range_empty_range_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    // no rows inserted — scan returns nothing
    let affected = c.copy_range(1, 2, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn copy_range_copies_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    // col_a values 10,20,30 all in [10,30]
    c.insert(1, vec![1, 10, 0]).unwrap();
    c.insert(2, vec![2, 20, 0]).unwrap();
    c.insert(3, vec![3, 30, 0]).unwrap();
    let affected = c.copy_range(1, 2, 10, 30).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[2], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[2], 20);
    assert_eq!(c.get(3).unwrap().unwrap()[2], 30);
}

#[test]
fn copy_range_only_in_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 0]).unwrap();
    c.insert(5, vec![5, 50, 0]).unwrap();
    // scan range [10,10] — only pk=1 col_a=10 qualifies
    c.copy_range(1, 2, 10, 10).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[2], 10);
    assert_eq!(c.get(5).unwrap().unwrap()[2], 0);
}

#[test]
fn copy_range_overwrites_existing() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42, 100]).unwrap();
    // col_a=42 in [42,42]
    c.copy_range(1, 2, 42, 42).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[2], 42);
}

#[test]
fn copy_range_same_col_is_noop() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 55, 0]).unwrap();
    // copy col_a → col_a
    let affected = c.copy_range(1, 1, 55, 55).unwrap();
    assert_eq!(affected, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 55);
}

#[test]
fn copy_range_zero_value_propagates() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 99]).unwrap();
    // col_a=0 in [0,0]
    c.copy_range(1, 2, 0, 0).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[2], 0);
}
