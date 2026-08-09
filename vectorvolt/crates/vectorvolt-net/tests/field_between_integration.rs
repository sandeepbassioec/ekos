//! Phase 53: FieldBetween integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "score".into(), "tag".into()]
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
fn field_between_empty_table_returns_no_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.field_between(1, 0, 100, 2, 0, 50).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn field_between_filter_excludes_out_of_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    // score=10 in scan range, tag=99 outside filter [0,50]
    c.insert(1, vec![1, 10, 99]).unwrap();
    let rows = c.field_between(1, 0, 100, 2, 0, 50).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn field_between_returns_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 5]).unwrap();
    c.insert(2, vec![2, 20, 99]).unwrap();
    c.insert(3, vec![3, 30, 15]).unwrap();
    // scan col1 [0,100], filter col2 [0,50] → rows 1 and 3
    let rows = c.field_between(1, 0, 100, 2, 0, 50).unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn field_between_scan_range_excludes_some() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 10]).unwrap(); // score=5, outside scan [10,100]
    c.insert(2, vec![2, 50, 10]).unwrap(); // score=50, inside scan
    let rows = c.field_between(1, 10, 100, 2, 0, 50).unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn field_between_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 5]).unwrap();
    c.field_between(1, 0, 100, 2, 0, 50).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
}

#[test]
fn field_between_all_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, i * 10, 5]).unwrap();
    }
    let rows = c.field_between(1, 0, 100, 2, 0, 10).unwrap();
    assert_eq!(rows.len(), 4);
}

#[test]
fn field_between_boundary_inclusive() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 0]).unwrap();
    c.insert(2, vec![2, 20, 50]).unwrap();
    let rows = c.field_between(1, 0, 100, 2, 0, 50).unwrap();
    assert_eq!(rows.len(), 2);
}
