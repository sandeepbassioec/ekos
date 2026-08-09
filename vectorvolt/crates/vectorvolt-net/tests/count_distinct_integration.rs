//! Phase 49: CountDistinct integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "tag".into()]
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
fn count_distinct_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.count_distinct(1, 0, u64::MAX).unwrap(), 0);
}

#[test]
fn count_distinct_all_same_is_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, 7]).unwrap();
    }
    assert_eq!(c.count_distinct(1, 0, 100).unwrap(), 1);
}

#[test]
fn count_distinct_all_different() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    assert_eq!(c.count_distinct(1, 0, 100).unwrap(), 5);
}

#[test]
fn count_distinct_some_duplicates() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.insert(2, vec![2, 1]).unwrap();
    c.insert(3, vec![3, 2]).unwrap();
    c.insert(4, vec![4, 3]).unwrap();
    c.insert(5, vec![5, 3]).unwrap();
    assert_eq!(c.count_distinct(1, 0, 10).unwrap(), 3); // 1, 2, 3
}

#[test]
fn count_distinct_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // tag=5, outside [10,50]
    c.insert(2, vec![2, 10]).unwrap(); // inside
    c.insert(3, vec![3, 10]).unwrap(); // duplicate 10 inside
    c.insert(4, vec![4, 50]).unwrap(); // inside
    // range [10, 50] → tags 10, 10, 50 → 2 distinct
    assert_eq!(c.count_distinct(1, 10, 50).unwrap(), 2);
}

#[test]
fn count_distinct_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.count_distinct(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn count_distinct_single_row_is_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    assert_eq!(c.count_distinct(1, 0, 100).unwrap(), 1);
}
