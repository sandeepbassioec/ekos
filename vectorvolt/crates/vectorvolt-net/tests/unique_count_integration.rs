//! Phase 68: UniqueCount integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "tag".into()]
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
fn unique_count_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.unique_count(1, 0, 100).unwrap(), 0);
}

#[test]
fn unique_count_all_same_tag() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    c.insert(3, vec![3, 5]).unwrap();
    assert_eq!(c.unique_count(1, 0, 100).unwrap(), 1);
}

#[test]
fn unique_count_all_distinct_tags() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.unique_count(1, 0, 100).unwrap(), 3);
}

#[test]
fn unique_count_skips_zeros() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    c.insert(3, vec![3, 7]).unwrap();
    assert_eq!(c.unique_count(1, 0, 100).unwrap(), 1);
}

#[test]
fn unique_count_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 50]).unwrap(); // inside [10,100]
    c.insert(3, vec![3, 50]).unwrap(); // same tag — still 1 unique
    assert_eq!(c.unique_count(1, 10, 100).unwrap(), 1);
}

#[test]
fn unique_count_mixed() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 3]).unwrap();
    c.insert(2, vec![2, 3]).unwrap();
    c.insert(3, vec![3, 7]).unwrap();
    c.insert(4, vec![4, 0]).unwrap();
    assert_eq!(c.unique_count(1, 0, 100).unwrap(), 2);
}
