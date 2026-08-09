//! Phase 64: MinRow integration tests.

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
fn min_row_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.min_row(1, 0, 100).unwrap().is_none());
}

#[test]
fn min_row_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let row = c.min_row(1, 0, 100).unwrap().unwrap();
    assert_eq!(row[1], 42);
}

#[test]
fn min_row_returns_lowest() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 80]).unwrap();
    let row = c.min_row(1, 0, 100).unwrap().unwrap();
    assert_eq!(row[1], 10);
}

#[test]
fn min_row_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 30]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    let row = c.min_row(1, 10, 100).unwrap().unwrap();
    assert_eq!(row[1], 20);
}

#[test]
fn min_row_does_not_modify_data() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.min_row(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn min_row_all_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, 42]).unwrap();
    }
    let row = c.min_row(1, 0, 100).unwrap().unwrap();
    assert_eq!(row[1], 42);
}

#[test]
fn min_row_returns_full_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(3, vec![3, 5]).unwrap();
    let row = c.min_row(1, 0, 100).unwrap().unwrap();
    assert_eq!(row[0], 3);
    assert_eq!(row[1], 5);
}
