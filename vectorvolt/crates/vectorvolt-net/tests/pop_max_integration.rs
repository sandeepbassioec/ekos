//! Phase 28: PopMax integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "priority".into()]
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
fn pop_max_empty_table_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.pop_max(0, 0, u64::MAX).unwrap().is_none());
}

#[test]
fn pop_max_single_row_returns_and_removes_it() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let row = c.pop_max(0, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(row[0], 1);
    assert!(c.get(1).unwrap().is_none());
}

#[test]
fn pop_max_returns_row_with_maximum_col_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 30]).unwrap();
    c.insert(2, vec![2, 90]).unwrap(); // max
    c.insert(3, vec![3, 20]).unwrap();
    let row = c.pop_max(1, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(row[0], 2);
    assert_eq!(row[1], 90);
    assert!(c.get(2).unwrap().is_none());
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(3).unwrap().is_some());
}

#[test]
fn pop_max_successive_calls_drain_in_descending_order() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for (pk, pri) in [(1u64, 50u64), (2, 10), (3, 30), (4, 20)] {
        c.insert(pk, vec![pk, pri]).unwrap();
    }
    let first = c.pop_max(1, 0, u64::MAX).unwrap().unwrap();
    let second = c.pop_max(1, 0, u64::MAX).unwrap().unwrap();
    let third = c.pop_max(1, 0, u64::MAX).unwrap().unwrap();
    let fourth = c.pop_max(1, 0, u64::MAX).unwrap().unwrap();
    assert!(c.pop_max(1, 0, u64::MAX).unwrap().is_none());
    assert_eq!(first[1], 50);
    assert_eq!(second[1], 30);
    assert_eq!(third[1], 20);
    assert_eq!(fourth[1], 10);
}

#[test]
fn pop_max_respects_range_bounds() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap(); // above range
    c.insert(2, vec![2, 25]).unwrap(); // in range — max
    c.insert(3, vec![3, 15]).unwrap(); // in range
    let row = c.pop_max(1, 10, 30).unwrap().unwrap();
    assert_eq!(row[1], 25);
    assert!(c.get(1).unwrap().is_some()); // outside range survives
}

#[test]
fn pop_max_out_of_range_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    assert!(c.pop_max(1, 100, 200).unwrap().is_none());
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(2).unwrap().is_some());
}

#[test]
fn pop_max_after_full_drain_returns_not_found() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    assert!(c.pop_max(1, 0, u64::MAX).unwrap().is_some());
    assert!(c.pop_max(1, 0, u64::MAX).unwrap().is_none());
}
