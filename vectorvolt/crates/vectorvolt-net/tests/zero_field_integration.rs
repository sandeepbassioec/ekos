//! Phase 58: ZeroField integration tests.

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
fn zero_field_empty_table_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.zero_field(1, 0, 100).unwrap(), 0);
}

#[test]
fn zero_field_zeroes_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    c.insert(2, vec![2, 75]).unwrap();
    let count = c.zero_field(1, 0, 100).unwrap();
    assert_eq!(count, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 0);
}

#[test]
fn zero_field_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // score=5, outside [10,100]
    c.insert(2, vec![2, 50]).unwrap();
    let count = c.zero_field(1, 10, 100).unwrap();
    assert_eq!(count, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5); // unchanged
    assert_eq!(c.get(2).unwrap().unwrap()[1], 0);
}

#[test]
fn zero_field_already_zero_is_counted() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let count = c.zero_field(1, 0, 100).unwrap();
    assert_eq!(count, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn zero_field_does_not_affect_out_of_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap();
    c.zero_field(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 200);
}

#[test]
fn zero_field_returns_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let count = c.zero_field(1, 0, 100).unwrap();
    assert_eq!(count, 5);
}

#[test]
fn zero_field_pk_col_becomes_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.zero_field(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}
