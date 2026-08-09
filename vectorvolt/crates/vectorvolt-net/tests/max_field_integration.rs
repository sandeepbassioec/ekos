//! Phase 46: MaxField integration tests.

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
fn max_field_absent_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.max_field(99, vec![1, 2]).unwrap().is_none());
}

#[test]
fn max_field_single_col() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42, 0, 0]).unwrap();
    assert_eq!(c.max_field(1, vec![1]).unwrap(), Some(42));
}

#[test]
fn max_field_picks_largest() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 99, 5]).unwrap();
    assert_eq!(c.max_field(1, vec![1, 2, 3]).unwrap(), Some(99));
}

#[test]
fn max_field_all_equal() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7, 7, 7]).unwrap();
    assert_eq!(c.max_field(1, vec![1, 2, 3]).unwrap(), Some(7));
}

#[test]
fn max_field_first_is_largest() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100, 50, 25]).unwrap();
    assert_eq!(c.max_field(1, vec![1, 2, 3]).unwrap(), Some(100));
}

#[test]
fn max_field_does_not_modify_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 3, 1, 2]).unwrap();
    c.max_field(1, vec![1, 2, 3]).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 3);
    assert_eq!(row[2], 1);
}

#[test]
fn max_field_u64_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1, u64::MAX, 0]).unwrap();
    assert_eq!(c.max_field(1, vec![1, 2, 3]).unwrap(), Some(u64::MAX));
}
