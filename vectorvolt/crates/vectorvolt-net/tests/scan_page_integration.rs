//! Phase 59: ScanPage integration tests.

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
fn scan_page_empty_table_returns_no_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.scan_page(1, 0, 100, 0, 10).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn scan_page_first_page() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let rows = c.scan_page(1, 0, 100, 0, 3).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn scan_page_offset_skips_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let rows = c.scan_page(1, 0, 100, 3, 10).unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn scan_page_offset_beyond_end_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=3 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let rows = c.scan_page(1, 0, 100, 10, 5).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn scan_page_limit_zero_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let rows = c.scan_page(1, 0, 100, 0, 0).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn scan_page_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.scan_page(1, 0, 100, 0, 10).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 99);
}

#[test]
fn scan_page_range_filter_respected() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.scan_page(1, 10, 100, 0, 10).unwrap();
    assert_eq!(rows.len(), 2);
}
