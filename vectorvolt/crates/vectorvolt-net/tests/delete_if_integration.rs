//! Phase 41: DeleteIf integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "state".into()]
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
fn delete_if_absent_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.delete_if(99, 1, 0).unwrap().is_none());
}

#[test]
fn delete_if_match_deletes_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.delete_if(1, 1, 42).unwrap(), Some(true));
    assert!(c.get(1).unwrap().is_none());
}

#[test]
fn delete_if_mismatch_does_not_delete() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.delete_if(1, 1, 99).unwrap(), Some(false));
    assert!(c.get(1).unwrap().is_some());
}

#[test]
fn delete_if_row_absent_after_delete() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7]).unwrap();
    c.delete_if(1, 1, 7).unwrap();
    assert!(c.get(1).unwrap().is_none());
}

#[test]
fn delete_if_second_call_after_delete_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.delete_if(1, 1, 5).unwrap();
    assert!(c.delete_if(1, 1, 5).unwrap().is_none());
}

#[test]
fn delete_if_zero_value_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    assert_eq!(c.delete_if(1, 1, 0).unwrap(), Some(true));
}

#[test]
fn delete_if_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.delete_if(1, 1, 10).unwrap();
    assert!(c.get(2).unwrap().is_some());
}
