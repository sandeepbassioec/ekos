//! Phase 36: IncrementMulti integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "a".into(), "b".into()]
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
fn increment_multi_absent_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.increment_multi(99, vec![(1, 5)]).unwrap().is_none());
}

#[test]
fn increment_multi_single_field() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();
    let new_vals = c.increment_multi(1, vec![(1, 5)]).unwrap().unwrap();
    assert_eq!(new_vals[1], 15);
    assert_eq!(new_vals[2], 20);
}

#[test]
fn increment_multi_two_fields() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();
    let new_vals = c.increment_multi(1, vec![(1, 3), (2, 7)]).unwrap().unwrap();
    assert_eq!(new_vals[1], 13);
    assert_eq!(new_vals[2], 27);
}

#[test]
fn increment_multi_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX - 1, 0]).unwrap();
    let new_vals = c.increment_multi(1, vec![(1, 5)]).unwrap().unwrap();
    assert_eq!(new_vals[1], u64::MAX);
}

#[test]
fn increment_multi_returns_full_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 0]).unwrap();
    let new_vals = c.increment_multi(1, vec![(1, 100)]).unwrap().unwrap();
    assert_eq!(new_vals.len(), 3);
    assert_eq!(new_vals[0], 1); // pk unchanged
}

#[test]
fn increment_multi_persists() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 5]).unwrap();
    c.increment_multi(1, vec![(1, 10), (2, 20)]).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 15);
    assert_eq!(row[2], 25);
}

#[test]
fn increment_multi_zero_delta_is_noop() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42, 0]).unwrap();
    let new_vals = c.increment_multi(1, vec![(1, 0)]).unwrap().unwrap();
    assert_eq!(new_vals[1], 42);
}
