//! Phase 109: BatchGetField integration tests.

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
fn batch_get_field_empty_list() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.batch_get_field(1, vec![]).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn batch_get_field_single_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let rows = c.batch_get_field(1, vec![1]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 1);
    assert_eq!(rows[0][1], 42);
}

#[test]
fn batch_get_field_multiple_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.batch_get_field(1, vec![1, 2, 3]).unwrap();
    assert_eq!(rows.len(), 3);
    let vals: Vec<u64> = rows.iter().map(|r| r[1]).collect();
    assert!(vals.contains(&10));
    assert!(vals.contains(&20));
    assert!(vals.contains(&30));
}

#[test]
fn batch_get_field_missing_pk_skipped() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    let rows = c.batch_get_field(1, vec![1, 99]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 1);
}

#[test]
fn batch_get_field_row_has_pk_and_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 77]).unwrap();
    let rows = c.batch_get_field(1, vec![7]).unwrap();
    assert_eq!(rows[0].len(), 2);
    assert_eq!(rows[0][0], 7); // pk
    assert_eq!(rows[0][1], 77); // value
}

#[test]
fn batch_get_field_zero_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let rows = c.batch_get_field(1, vec![1]).unwrap();
    assert_eq!(rows[0][1], 0);
}
