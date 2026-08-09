//! Phase 71: MoveField integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "credits".into()]
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
fn move_field_missing_src_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    assert!(c.move_field(99, 2, 1).unwrap().is_none());
}

#[test]
fn move_field_missing_dst_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    assert!(c.move_field(1, 99, 1).unwrap().is_none());
}

#[test]
fn move_field_transfers_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 40]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    let old = c.move_field(1, 2, 1).unwrap().unwrap();
    assert_eq!(old, 40);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 50);
}

#[test]
fn move_field_zeroes_source() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    c.move_field(1, 2, 1).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn move_field_accumulates_on_dst() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 30]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.move_field(1, 2, 1).unwrap();
    assert_eq!(c.get(2).unwrap().unwrap()[1], 50);
}

#[test]
fn move_field_returns_moved_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    let val = c.move_field(1, 2, 1).unwrap().unwrap();
    assert_eq!(val, 77);
}
