//! Phase 42: SetFieldIf integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "status".into()]
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
fn set_field_if_absent_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.set_field_if(99, 1, 0, 1).unwrap().is_none());
}

#[test]
fn set_field_if_match_updates_field() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    assert_eq!(c.set_field_if(1, 1, 10, 20).unwrap(), Some(true));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 20);
}

#[test]
fn set_field_if_mismatch_no_update() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    assert_eq!(c.set_field_if(1, 1, 99, 20).unwrap(), Some(false));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
}

#[test]
fn set_field_if_update_to_same_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.set_field_if(1, 1, 42, 42).unwrap(), Some(true));
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn set_field_if_chained_state_machine() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap(); // state 0
    assert_eq!(c.set_field_if(1, 1, 0, 1).unwrap(), Some(true)); // 0 -> 1
    assert_eq!(c.set_field_if(1, 1, 1, 2).unwrap(), Some(true)); // 1 -> 2
    assert_eq!(c.set_field_if(1, 1, 1, 99).unwrap(), Some(false)); // still 2
    assert_eq!(c.get(1).unwrap().unwrap()[1], 2);
}

#[test]
fn set_field_if_does_not_change_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(5, vec![5, 7]).unwrap();
    c.set_field_if(5, 1, 7, 8).unwrap();
    assert_eq!(c.get(5).unwrap().unwrap()[0], 5);
}

#[test]
fn set_field_if_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.set_field_if(1, 1, 10, 99).unwrap();
    assert_eq!(c.get(2).unwrap().unwrap()[1], 10);
}
