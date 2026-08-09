//! Phase 39: Clamp integration tests.

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
fn clamp_absent_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.clamp(99, 1, 0, 100).unwrap().is_none());
}

#[test]
fn clamp_value_below_lo() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    assert_eq!(c.clamp(1, 1, 10, 100).unwrap(), Some(10));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
}

#[test]
fn clamp_value_above_hi() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap();
    assert_eq!(c.clamp(1, 1, 10, 100).unwrap(), Some(100));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
}

#[test]
fn clamp_value_within_range_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    assert_eq!(c.clamp(1, 1, 10, 100).unwrap(), Some(50));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
}

#[test]
fn clamp_exact_lo_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    assert_eq!(c.clamp(1, 1, 10, 100).unwrap(), Some(10));
}

#[test]
fn clamp_exact_hi_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    assert_eq!(c.clamp(1, 1, 10, 100).unwrap(), Some(100));
}

#[test]
fn clamp_persists_to_store() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 999]).unwrap();
    c.clamp(1, 1, 0, 50).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 50);
}
