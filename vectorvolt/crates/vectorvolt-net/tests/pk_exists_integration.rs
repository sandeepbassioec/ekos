//! Phase 35: PkExists integration tests.

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
fn pk_exists_absent_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(!c.pk_exists(99).unwrap());
}

#[test]
fn pk_exists_present_returns_true() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert!(c.pk_exists(1).unwrap());
}

#[test]
fn pk_exists_after_delete_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.delete(1).unwrap();
    assert!(!c.pk_exists(1).unwrap());
}

#[test]
fn pk_exists_does_not_affect_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.pk_exists(1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn pk_exists_multiple_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(3, vec![3, 0]).unwrap();
    assert!(c.pk_exists(1).unwrap());
    assert!(!c.pk_exists(2).unwrap());
    assert!(c.pk_exists(3).unwrap());
}

#[test]
fn pk_exists_after_upsert_returns_true() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.upsert(5, vec![5, 99]).unwrap();
    assert!(c.pk_exists(5).unwrap());
}

#[test]
fn pk_exists_max_minus_one_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let pk = u64::MAX - 1;
    assert!(!c.pk_exists(pk).unwrap());
    c.insert(pk, vec![pk, 0]).unwrap();
    assert!(c.pk_exists(pk).unwrap());
}
