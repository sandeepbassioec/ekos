//! Phase 50: Median integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
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
fn median_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.median(1, 0, 100).unwrap().is_none());
}

#[test]
fn median_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.median(1, 0, 100).unwrap(), Some(42));
}

#[test]
fn median_odd_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 30]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    // sorted: [10,20,30] → mid=1 → 20
    assert_eq!(c.median(1, 0, 100).unwrap(), Some(20));
}

#[test]
fn median_even_count_upper_half() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    // sorted: [10,20] → mid=1 → 20
    assert_eq!(c.median(1, 0, 100).unwrap(), Some(20));
}

#[test]
fn median_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();   // outside [10,100]
    c.insert(2, vec![2, 30]).unwrap();
    c.insert(3, vec![3, 50]).unwrap();
    c.insert(4, vec![4, 70]).unwrap();
    // range [10,100]: [30,50,70] → mid=1 → 50
    assert_eq!(c.median(1, 10, 100).unwrap(), Some(50));
}

#[test]
fn median_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.median(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn median_all_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, 42]).unwrap();
    }
    assert_eq!(c.median(1, 0, 100).unwrap(), Some(42));
}
