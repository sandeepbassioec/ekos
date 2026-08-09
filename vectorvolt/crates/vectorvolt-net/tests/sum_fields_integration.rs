//! Phase 45: SumFields integration tests.

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
fn sum_fields_absent_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.sum_fields(99, vec![1, 2]).unwrap().is_none());
}

#[test]
fn sum_fields_single_col() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    assert_eq!(c.sum_fields(1, vec![1]).unwrap(), Some(10));
}

#[test]
fn sum_fields_two_cols() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    assert_eq!(c.sum_fields(1, vec![1, 2]).unwrap(), Some(30));
}

#[test]
fn sum_fields_all_three_data_cols() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    assert_eq!(c.sum_fields(1, vec![1, 2, 3]).unwrap(), Some(60));
}

#[test]
fn sum_fields_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX, u64::MAX, 0]).unwrap();
    assert_eq!(c.sum_fields(1, vec![1, 2]).unwrap(), Some(u64::MAX));
}

#[test]
fn sum_fields_includes_pk_col() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 10, 0, 0]).unwrap();
    // col 0 is pk = 5
    assert_eq!(c.sum_fields(5, vec![0, 1]).unwrap(), Some(15));
}

#[test]
fn sum_fields_does_not_modify_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7, 3, 0]).unwrap();
    c.sum_fields(1, vec![1, 2]).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 7);
    assert_eq!(row[2], 3);
}
