//! Phase 48: BatchIncrementField integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "counter".into()]
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
fn batch_increment_field_empty_pks_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_increment_field(1, 5, vec![]).unwrap(), 0);
}

#[test]
fn batch_increment_field_missing_pks_skipped() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_increment_field(1, 5, vec![99, 100]).unwrap(), 0);
}

#[test]
fn batch_increment_field_updates_all_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let count = c.batch_increment_field(1, 5, vec![1, 2, 3]).unwrap();
    assert_eq!(count, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 15);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 25);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 35);
}

#[test]
fn batch_increment_field_mixed_present_absent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(3, vec![3, 300]).unwrap();
    let count = c.batch_increment_field(1, 10, vec![1, 2, 3]).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn batch_increment_field_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX - 1]).unwrap();
    c.batch_increment_field(1, 5, vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], u64::MAX);
}

#[test]
fn batch_increment_field_zero_delta_is_noop() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.batch_increment_field(1, 0, vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn batch_increment_field_returns_updated_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, 0]).unwrap();
    }
    let count = c.batch_increment_field(1, 1, vec![1, 2, 3, 4, 5]).unwrap();
    assert_eq!(count, 5);
}
