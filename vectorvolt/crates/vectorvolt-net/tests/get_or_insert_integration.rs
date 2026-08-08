//! Phase 31: GetOrInsert integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
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
fn get_or_insert_inserts_when_absent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let row = c.get_or_insert(1, vec![1, 42]).unwrap();
    assert_eq!(row, vec![1, 42]);
    assert!(c.get(1).unwrap().is_some());
}

#[test]
fn get_or_insert_returns_existing_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    let row = c.get_or_insert(1, vec![1, 0]).unwrap();
    assert_eq!(row[1], 99); // existing value, not the default 0
}

#[test]
fn get_or_insert_does_not_overwrite_existing() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.get_or_insert(1, vec![1, 0]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn get_or_insert_idempotent_on_absent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let first = c.get_or_insert(1, vec![1, 5]).unwrap();
    let second = c.get_or_insert(1, vec![1, 99]).unwrap();
    assert_eq!(first, second); // second call returns what first created
}

#[test]
fn get_or_insert_multiple_distinct_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let r1 = c.get_or_insert(1, vec![1, 10]).unwrap();
    let r2 = c.get_or_insert(2, vec![2, 20]).unwrap();
    assert_eq!(r1[1], 10);
    assert_eq!(r2[1], 20);
}

#[test]
fn get_or_insert_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    c.get_or_insert(2, vec![2, 99]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
}

#[test]
fn get_or_insert_returns_row_with_pk_as_values_0() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let row = c.get_or_insert(7, vec![7, 123]).unwrap();
    assert_eq!(row[0], 7);
    assert_eq!(row[1], 123);
}
