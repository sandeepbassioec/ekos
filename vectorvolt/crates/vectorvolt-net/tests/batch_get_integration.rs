//! Phase 61: BatchGet integration tests.

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
fn batch_get_empty_pks_returns_no_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.batch_get(vec![]).unwrap().is_empty());
}

#[test]
fn batch_get_all_absent_returns_no_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.batch_get(vec![1, 2, 3]).unwrap().is_empty());
}

#[test]
fn batch_get_all_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.batch_get(vec![1, 2, 3]).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn batch_get_partial_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rows = c.batch_get(vec![1, 2, 3]).unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn batch_get_returns_correct_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 99]).unwrap();
    let rows = c.batch_get(vec![5]).unwrap();
    assert_eq!(rows[0][1], 99);
}

#[test]
fn batch_get_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.batch_get(vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn batch_get_after_delete_skips_deleted() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.delete(1).unwrap();
    let rows = c.batch_get(vec![1, 2]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 2);
}
