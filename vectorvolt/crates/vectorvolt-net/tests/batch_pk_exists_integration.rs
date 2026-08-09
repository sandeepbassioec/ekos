//! Phase 52: BatchPkExists integration tests.

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
fn batch_pk_exists_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_pk_exists(vec![]).unwrap(), 0);
}

#[test]
fn batch_pk_exists_all_absent() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_pk_exists(vec![1, 2, 3]).unwrap(), 0);
}

#[test]
fn batch_pk_exists_some_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.batch_pk_exists(vec![1, 2, 3]).unwrap(), 2);
}

#[test]
fn batch_pk_exists_all_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    assert_eq!(c.batch_pk_exists(vec![1, 2, 3, 4, 5]).unwrap(), 5);
}

#[test]
fn batch_pk_exists_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.batch_pk_exists(vec![1]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 99);
}

#[test]
fn batch_pk_exists_after_delete_is_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.delete(1).unwrap();
    assert_eq!(c.batch_pk_exists(vec![1]).unwrap(), 0);
}

#[test]
fn batch_pk_exists_single_present() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 0]).unwrap();
    assert_eq!(c.batch_pk_exists(vec![42]).unwrap(), 1);
}
