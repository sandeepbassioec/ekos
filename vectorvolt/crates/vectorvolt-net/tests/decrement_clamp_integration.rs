//! Phase 30: DecrementClamp integration tests.

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
fn decrement_clamp_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.decrement_clamp(99, 1, 1).unwrap().is_none());
}

#[test]
fn decrement_clamp_normal_subtraction() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    let v = c.decrement_clamp(1, 1, 30).unwrap().unwrap();
    assert_eq!(v, 70);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 70);
}

#[test]
fn decrement_clamp_clamps_at_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    let v = c.decrement_clamp(1, 1, 999).unwrap().unwrap();
    assert_eq!(v, 0);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}

#[test]
fn decrement_clamp_exact_to_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    let v = c.decrement_clamp(1, 1, 50).unwrap().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn decrement_clamp_from_zero_stays_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let v = c.decrement_clamp(1, 1, 1).unwrap().unwrap();
    assert_eq!(v, 0);
}

#[test]
fn decrement_clamp_successive_calls_accumulate() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.decrement_clamp(1, 1, 30).unwrap();
    c.decrement_clamp(1, 1, 30).unwrap();
    let v = c.decrement_clamp(1, 1, 30).unwrap().unwrap();
    assert_eq!(v, 10);
}

#[test]
fn decrement_clamp_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    c.decrement_clamp(1, 1, 50).unwrap();
    assert_eq!(c.get(2).unwrap().unwrap()[1], 200);
}
