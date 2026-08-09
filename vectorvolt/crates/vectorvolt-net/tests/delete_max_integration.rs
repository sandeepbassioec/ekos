//! Phase 67: DeleteMax integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "score".into()]
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
fn delete_max_empty_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.delete_max(1, 0, 100).unwrap().is_none());
}

#[test]
fn delete_max_single_row_deletes_it() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    let pk = c.delete_max(1, 0, 100).unwrap().unwrap();
    assert_eq!(pk, 1);
    assert!(c.get(1).unwrap().is_none());
}

#[test]
fn delete_max_removes_highest() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 80]).unwrap();
    let pk = c.delete_max(1, 0, 100).unwrap().unwrap();
    assert_eq!(pk, 3); // pk=3 has highest score=80
    assert!(c.get(3).unwrap().is_none());
    assert!(c.get(1).unwrap().is_some());
    assert!(c.get(2).unwrap().is_some());
}

#[test]
fn delete_max_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 30]).unwrap();
    c.insert(3, vec![3, 80]).unwrap();
    c.delete_max(1, 10, 100).unwrap();
    assert!(c.get(1).unwrap().is_some()); // unaffected
    assert!(c.get(3).unwrap().is_none()); // max in range was 80
}

#[test]
fn delete_max_returns_pk() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(7, vec![7, 99]).unwrap();
    let pk = c.delete_max(1, 0, 100).unwrap().unwrap();
    assert_eq!(pk, 7);
}

#[test]
fn delete_max_successive_removes_in_order() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 30]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    let p1 = c.delete_max(1, 0, 100).unwrap().unwrap();
    let p2 = c.delete_max(1, 0, 100).unwrap().unwrap();
    let p3 = c.delete_max(1, 0, 100).unwrap().unwrap();
    assert_eq!(p1, 1);
    assert_eq!(p2, 3);
    assert_eq!(p3, 2);
}

#[test]
fn delete_max_does_not_touch_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.delete_max(1, 0, 100).unwrap();
    assert!(c.get(1).unwrap().is_some());
}
