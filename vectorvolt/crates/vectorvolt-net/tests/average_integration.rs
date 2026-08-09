//! Phase 40: Average integration tests.

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
fn average_empty_range_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.average(1, 0, 100).unwrap().is_none());
}

#[test]
fn average_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 40]).unwrap();
    let (sum, count) = c.average(1, 0, 100).unwrap().unwrap();
    assert_eq!(sum, 40);
    assert_eq!(count, 1);
}

#[test]
fn average_multiple_rows_sum_and_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let (sum, count) = c.average(1, 0, 100).unwrap().unwrap();
    assert_eq!(sum, 60);
    assert_eq!(count, 3);
}

#[test]
fn average_range_filter() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    c.insert(3, vec![3, 500]).unwrap();
    // range [10, 100] catches only pk=2
    let (sum, count) = c.average(1, 10, 100).unwrap().unwrap();
    assert_eq!(sum, 50);
    assert_eq!(count, 1);
}

#[test]
fn average_sum_saturation() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, u64::MAX]).unwrap();
    c.insert(2, vec![2, u64::MAX]).unwrap();
    let (sum, count) = c.average(1, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(sum, u64::MAX); // saturated
    assert_eq!(count, 2);
}

#[test]
fn average_returns_correct_count() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10]).unwrap();
    }
    let (_, count) = c.average(1, 0, u64::MAX).unwrap().unwrap();
    assert_eq!(count, 5);
}

#[test]
fn average_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.average(1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}
