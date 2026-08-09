//! Phase 70: BottomN integration tests.

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
fn bottom_n_empty_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.bottom_n(1, 3, 0, 100).unwrap().is_empty());
}

#[test]
fn bottom_n_returns_lowest_first() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.bottom_n(1, 3, 0, 100).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][1], 10);
    assert_eq!(rows[1][1], 30);
    assert_eq!(rows[2][1], 50);
}

#[test]
fn bottom_n_limits_to_n() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let rows = c.bottom_n(1, 2, 0, 100).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][1], 10);
    assert_eq!(rows[1][1], 20);
}

#[test]
fn bottom_n_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap(); // outside [0, 100]
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 40]).unwrap();
    let rows = c.bottom_n(1, 3, 0, 100).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][1], 20);
}

#[test]
fn bottom_n_returns_full_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(4, vec![4, 1]).unwrap();
    let rows = c.bottom_n(1, 1, 0, 100).unwrap();
    assert_eq!(rows[0][0], 4);
    assert_eq!(rows[0][1], 1);
}

#[test]
fn bottom_n_n_larger_than_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 3]).unwrap();
    let rows = c.bottom_n(1, 100, 0, 100).unwrap();
    assert_eq!(rows.len(), 2);
}
