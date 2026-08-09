//! Phase 88: MultiColumnSum integration tests.

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
fn multi_column_sum_empty_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let total = c.multi_column_sum(vec![], 1).unwrap();
    assert_eq!(total, 0);
}

#[test]
fn multi_column_sum_single_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let total = c.multi_column_sum(vec![1], 1).unwrap();
    assert_eq!(total, 42);
}

#[test]
fn multi_column_sum_multiple_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let total = c.multi_column_sum(vec![1, 2, 3], 1).unwrap();
    assert_eq!(total, 60);
}

#[test]
fn multi_column_sum_missing_pk_contributes_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    // pk=99 missing → treated as 0
    let total = c.multi_column_sum(vec![1, 99], 1).unwrap();
    assert_eq!(total, 50);
}

#[test]
fn multi_column_sum_zero_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    let total = c.multi_column_sum(vec![1, 2], 1).unwrap();
    assert_eq!(total, 0);
}

#[test]
fn multi_column_sum_partial_exists() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 100]).unwrap();
    c.insert(20, vec![20, 200]).unwrap();
    let total = c.multi_column_sum(vec![10, 15, 20], 1).unwrap();
    assert_eq!(total, 300);
}
