//! Phase 87: SetFieldMany integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "status".into()]
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
fn set_field_many_empty_list() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.set_field_many(1, 99, vec![]).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn set_field_many_single_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let affected = c.set_field_many(1, 42, vec![1]).unwrap();
    assert_eq!(affected, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn set_field_many_multiple_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    c.insert(3, vec![3, 0]).unwrap();
    let affected = c.set_field_many(1, 7, vec![1, 2, 3]).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 7);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 7);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 7);
}

#[test]
fn set_field_many_missing_pk_skipped() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    // pk=99 doesn't exist
    let affected = c.set_field_many(1, 5, vec![1, 99]).unwrap();
    assert_eq!(affected, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 5);
}

#[test]
fn set_field_many_zero_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.set_field_many(1, 0, vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn set_field_many_partial_exists() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(10, vec![10, 1]).unwrap();
    c.insert(20, vec![20, 2]).unwrap();
    let affected = c.set_field_many(1, 100, vec![10, 15, 20]).unwrap();
    // pk=15 missing → 2 updated
    assert_eq!(affected, 2);
    assert_eq!(c.get(10).unwrap().unwrap()[1], 100);
    assert_eq!(c.get(20).unwrap().unwrap()[1], 100);
}
