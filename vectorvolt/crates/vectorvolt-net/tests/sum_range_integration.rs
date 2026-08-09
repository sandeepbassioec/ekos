//! Phase 98: SumRange integration tests.

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
fn sum_range_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.sum_range(1, 100, 200).unwrap(), 0);
}

#[test]
fn sum_range_all() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.sum_range(1, 10, 30).unwrap(), 60);
}

#[test]
fn sum_range_partial() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 15]).unwrap();
    c.insert(3, vec![3, 100]).unwrap();
    // scan [5,15]: 5+15=20
    assert_eq!(c.sum_range(1, 5, 15).unwrap(), 20);
}

#[test]
fn sum_range_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.sum_range(1, 42, 42).unwrap(), 42);
}

#[test]
fn sum_range_zero_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    assert_eq!(c.sum_range(1, 0, 0).unwrap(), 0);
}

#[test]
fn sum_range_saturates() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    c.insert(2, vec![2, u64::MAX]).unwrap();
    let s = c.sum_range(1, u64::MAX, u64::MAX).unwrap();
    assert_eq!(s, u64::MAX);
}
