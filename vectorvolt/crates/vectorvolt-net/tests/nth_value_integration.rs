//! Phase 85: NthValue integration tests.

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
fn nth_value_empty_range_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.nth_value(1, 0, 100, 200).unwrap().is_none());
}

#[test]
fn nth_value_zero_returns_first() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    // n=0 → first sorted element = 10
    let v = c.nth_value(1, 0, 10, 30).unwrap().unwrap();
    assert_eq!(v, 10);
}

#[test]
fn nth_value_last_element() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let v = c.nth_value(1, 2, 10, 30).unwrap().unwrap();
    assert_eq!(v, 30);
}

#[test]
fn nth_value_middle_element() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let v = c.nth_value(1, 1, 10, 30).unwrap().unwrap();
    assert_eq!(v, 20);
}

#[test]
fn nth_value_out_of_bounds_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    // n=5 but only 1 row → None
    assert!(c.nth_value(1, 5, 10, 10).unwrap().is_none());
}

#[test]
fn nth_value_single_row_n0() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    let v = c.nth_value(1, 0, 77, 77).unwrap().unwrap();
    assert_eq!(v, 77);
}
