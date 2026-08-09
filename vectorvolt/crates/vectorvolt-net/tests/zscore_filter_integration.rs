//! Phase 90: ZScoreFilter integration tests.

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
fn zscore_filter_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.zscore_filter(1, 100, 200).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn zscore_filter_all_same_returns_all() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    // stddev=0 when all values equal → all within 2σ
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    let rows = c.zscore_filter(1, 10, 10).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn zscore_filter_single_row_returned() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    let rows = c.zscore_filter(1, 50, 50).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 1);
}

#[test]
fn zscore_filter_returns_within_2sigma() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    // tightly clustered; all rows are within 2σ of each other
    c.insert(1, vec![1, 98]).unwrap();
    c.insert(2, vec![2, 100]).unwrap();
    c.insert(3, vec![3, 102]).unwrap();
    let rows = c.zscore_filter(1, 98, 102).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn zscore_filter_returns_pk_and_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 50]).unwrap();
    let rows = c.zscore_filter(1, 50, 50).unwrap();
    assert_eq!(rows[0][0], 5); // pk
    assert_eq!(rows[0][1], 50); // value
}

#[test]
fn zscore_filter_two_row_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    // mean=15, stddev~=5; both within 2σ
    let rows = c.zscore_filter(1, 10, 20).unwrap();
    assert_eq!(rows.len(), 2);
}
