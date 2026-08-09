//! Phase 47: FilterScan integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "category".into(), "score".into()]
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
fn filter_scan_empty_table_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.filter_scan(2, 0, 100, 1, 5).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn filter_scan_returns_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1, 10]).unwrap(); // category=1, score=10
    c.insert(2, vec![2, 2, 20]).unwrap(); // category=2, score=20
    c.insert(3, vec![3, 1, 30]).unwrap(); // category=1, score=30
    // scan on score [0,100], filter category==1
    let rows = c.filter_scan(2, 0, 100, 1, 1).unwrap();
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(row[1], 1);
    }
}

#[test]
fn filter_scan_no_match_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1, 10]).unwrap();
    c.insert(2, vec![2, 1, 20]).unwrap();
    let rows = c.filter_scan(2, 0, 100, 1, 99).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn filter_scan_range_restricts_results() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1, 5]).unwrap();  // score=5 outside [10,100]
    c.insert(2, vec![2, 1, 50]).unwrap(); // score=50 inside
    let rows = c.filter_scan(2, 10, 100, 1, 1).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 2);
}

#[test]
fn filter_scan_returns_all_columns() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 3, 42]).unwrap();
    let rows = c.filter_scan(2, 0, 100, 1, 3).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 7);
    assert_eq!(rows[0][1], 3);
    assert_eq!(rows[0][2], 42);
}

#[test]
fn filter_scan_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 50]).unwrap();
    c.filter_scan(2, 0, 100, 1, 5).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[2], 50);
}

#[test]
fn filter_scan_all_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, 7, i * 10]).unwrap();
    }
    let rows = c.filter_scan(2, 0, 100, 1, 7).unwrap();
    assert_eq!(rows.len(), 4);
}
