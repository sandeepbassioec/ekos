//! Phase 44: Percentile integration tests.

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
fn percentile_empty_range_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.percentile(1, 0, 100, 50).unwrap().is_none());
}

#[test]
fn percentile_50_of_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.percentile(1, 0, 100, 50).unwrap(), Some(42));
}

#[test]
fn percentile_0_is_minimum() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap(); // scores: 10,20,30,40,50
    }
    assert_eq!(c.percentile(1, 0, 100, 0).unwrap(), Some(10));
}

#[test]
fn percentile_100_is_maximum() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    assert_eq!(c.percentile(1, 0, 100, 100).unwrap(), Some(50));
}

#[test]
fn percentile_50_of_four_returns_middle() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, i * 10]).unwrap(); // scores: 10,20,30,40
    }
    // idx = 50 * 3 / 100 = 1 → sorted[1] = 20
    assert_eq!(c.percentile(1, 0, 100, 50).unwrap(), Some(20));
}

#[test]
fn percentile_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.percentile(1, 0, 100, 50).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn percentile_range_filter_applies() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.insert(3, vec![3, 500]).unwrap();
    // range [10, 100] only hits pk=2 (score=50)
    assert_eq!(c.percentile(1, 10, 100, 50).unwrap(), Some(50));
}
