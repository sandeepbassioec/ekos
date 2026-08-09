//! Phase 37: CopyRow integration tests.

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
fn copy_row_absent_source_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(!c.copy_row(99, 1).unwrap());
}

#[test]
fn copy_row_copies_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert!(c.copy_row(1, 2).unwrap());
    let dst = c.get(2).unwrap().unwrap();
    assert_eq!(dst[1], 42);
    assert_eq!(dst[0], 2); // pk updated to dst
}

#[test]
fn copy_row_source_still_exists() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 99]).unwrap();
    c.copy_row(1, 2).unwrap();
    assert!(c.get(1).unwrap().is_some());
}

#[test]
fn copy_row_overwrites_existing_dst() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 99]).unwrap();
    c.copy_row(1, 2).unwrap();
    let dst = c.get(2).unwrap().unwrap();
    assert_eq!(dst[1], 10);
}

#[test]
fn copy_row_dst_pk_field_updated() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 55]).unwrap();
    c.copy_row(1, 7).unwrap();
    let dst = c.get(7).unwrap().unwrap();
    assert_eq!(dst[0], 7);
}

#[test]
fn copy_row_returns_true_on_success() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 1]).unwrap();
    assert!(c.copy_row(5, 10).unwrap());
}

#[test]
fn copy_row_does_not_affect_source_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.copy_row(1, 2).unwrap();
    let src = c.get(1).unwrap().unwrap();
    assert_eq!(src[1], 77);
}
