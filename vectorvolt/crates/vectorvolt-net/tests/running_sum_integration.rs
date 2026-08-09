//! Phase 91: RunningSum integration tests.

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
fn running_sum_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.running_sum(1, 100, 200).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn running_sum_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let rows = c.running_sum(1, 42, 42).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][2], 42); // running_sum after single row = 42
}

#[test]
fn running_sum_accumulates() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.running_sum(1, 10, 30).unwrap();
    assert_eq!(rows.len(), 3);
    // running sum values are 10, 30, 60 (in scan order)
    let sums: Vec<u64> = rows.iter().map(|r| r[2]).collect();
    assert!(sums.contains(&10));
    assert!(sums.contains(&30));
    assert!(sums.contains(&60));
}

#[test]
fn running_sum_row_format() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 5]).unwrap();
    let rows = c.running_sum(1, 5, 5).unwrap();
    assert_eq!(rows[0].len(), 3);
    assert_eq!(rows[0][0], 7); // pk
    assert_eq!(rows[0][1], 5); // value
    assert_eq!(rows[0][2], 5); // running_sum
}

#[test]
fn running_sum_partial_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    // scan [10,10]: only pk=1 qualifies
    let rows = c.running_sum(1, 10, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][2], 10);
}

#[test]
fn running_sum_zero_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    let rows = c.running_sum(1, 0, 0).unwrap();
    assert_eq!(rows.len(), 2);
    for r in &rows {
        assert_eq!(r[2], 0);
    }
}
