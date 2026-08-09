//! Phase 62: SetFieldRange integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "status".into()]
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
fn set_field_range_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.set_field_range(1, 9, 0, 100).unwrap(), 0);
}

#[test]
fn set_field_range_updates_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    let count = c.set_field_range(1, 99, 0, 100).unwrap();
    assert_eq!(count, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 99);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 99);
}

#[test]
fn set_field_range_respects_scan_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // status=5, outside scan [10,100]
    c.insert(2, vec![2, 50]).unwrap();
    c.set_field_range(1, 0, 10, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5); // unchanged
    assert_eq!(c.get(2).unwrap().unwrap()[1], 0);
}

#[test]
fn set_field_range_returns_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let count = c.set_field_range(1, 7, 0, 100).unwrap();
    assert_eq!(count, 5);
}

#[test]
fn set_field_range_to_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.set_field_range(1, 0, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn set_field_range_idempotent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.set_field_range(1, 99, 0, 100).unwrap();
    c.set_field_range(1, 99, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 99);
}

#[test]
fn set_field_range_does_not_affect_other_cols() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.set_field_range(1, 55, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[0], 1); // pk unchanged
}
