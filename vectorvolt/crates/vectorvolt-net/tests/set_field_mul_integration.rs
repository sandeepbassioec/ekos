//! Phase 95: SetFieldMul integration tests.

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
fn set_field_mul_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.set_field_mul(99, 1, 2).unwrap().is_none());
}

#[test]
fn set_field_mul_multiplies_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    let new_val = c.set_field_mul(1, 1, 3).unwrap().unwrap();
    assert_eq!(new_val, 15);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 15);
}

#[test]
fn set_field_mul_factor_zero_zeroes() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let new_val = c.set_field_mul(1, 1, 0).unwrap().unwrap();
    assert_eq!(new_val, 0);
}

#[test]
fn set_field_mul_factor_one_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    let new_val = c.set_field_mul(1, 1, 1).unwrap().unwrap();
    assert_eq!(new_val, 77);
}

#[test]
fn set_field_mul_cumulative() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.set_field_mul(1, 1, 2).unwrap();
    c.set_field_mul(1, 1, 2).unwrap();
    let val = c.set_field_mul(1, 1, 2).unwrap().unwrap();
    assert_eq!(val, 8);
}

#[test]
fn set_field_mul_saturates_at_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    let val = c.set_field_mul(1, 1, 2).unwrap().unwrap();
    assert_eq!(val, u64::MAX);
}
