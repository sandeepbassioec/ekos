//! Phase 56: MultiGet integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "a".into(), "b".into(), "c".into()]
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
fn multi_get_absent_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.multi_get(99, vec![1, 2]).unwrap().is_none());
}

#[test]
fn multi_get_single_col() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    let result = c.multi_get(1, vec![2]).unwrap().unwrap();
    assert_eq!(result, vec![20]);
}

#[test]
fn multi_get_multiple_cols() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    let result = c.multi_get(1, vec![1, 3]).unwrap().unwrap();
    assert_eq!(result, vec![10, 30]);
}

#[test]
fn multi_get_all_cols() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    let result = c.multi_get(1, vec![0, 1, 2, 3]).unwrap().unwrap();
    assert_eq!(result, vec![1, 10, 20, 30]);
}

#[test]
fn multi_get_empty_cols_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    let result = c.multi_get(1, vec![]).unwrap().unwrap();
    assert!(result.is_empty());
}

#[test]
fn multi_get_does_not_modify_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42, 0, 0]).unwrap();
    c.multi_get(1, vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn multi_get_out_of_bounds_col_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    let result = c.multi_get(1, vec![99]).unwrap().unwrap();
    assert_eq!(result, vec![0]);
}
