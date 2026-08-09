//! Phase 96: SwapPkFields integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "col_a".into(), "col_b".into()]
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
fn swap_pk_fields_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.swap_pk_fields(99, 1, 2).unwrap().is_none());
}

#[test]
fn swap_pk_fields_swaps_columns() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();
    c.swap_pk_fields(1, 1, 2).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 20);
    assert_eq!(row[2], 10);
}

#[test]
fn swap_pk_fields_idempotent_twice() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 9]).unwrap();
    c.swap_pk_fields(1, 1, 2).unwrap();
    c.swap_pk_fields(1, 1, 2).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 5);
    assert_eq!(row[2], 9);
}

#[test]
fn swap_pk_fields_same_col_no_change() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7, 3]).unwrap();
    c.swap_pk_fields(1, 1, 1).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 7);
}

#[test]
fn swap_pk_fields_zero_and_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 99]).unwrap();
    c.swap_pk_fields(1, 1, 2).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 99);
    assert_eq!(row[2], 0);
}

#[test]
fn swap_pk_fields_returns_some() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();
    let result = c.swap_pk_fields(1, 1, 2).unwrap();
    assert!(result.is_some());
}
