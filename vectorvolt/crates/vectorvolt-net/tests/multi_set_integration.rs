//! Phase 29: MultiSet integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "a".into(), "b".into(), "c".into()]
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
fn multi_set_missing_pk_returns_false() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let ok = c.multi_set(99, vec![(1, 42)]).unwrap();
    assert!(!ok);
}

#[test]
fn multi_set_updates_single_column() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    assert!(c.multi_set(1, vec![(1, 99)]).unwrap());
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 99);
    assert_eq!(row[2], 20); // unchanged
    assert_eq!(row[3], 30); // unchanged
}

#[test]
fn multi_set_updates_multiple_columns() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1, 2, 3]).unwrap();
    assert!(c.multi_set(1, vec![(1, 100), (2, 200), (3, 300)]).unwrap());
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 100);
    assert_eq!(row[2], 200);
    assert_eq!(row[3], 300);
}

#[test]
fn multi_set_empty_pairs_is_no_op() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 7, 8, 9]).unwrap();
    assert!(c.multi_set(1, vec![]).unwrap());
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 7);
    assert_eq!(row[2], 8);
    assert_eq!(row[3], 9);
}

#[test]
fn multi_set_last_write_wins_for_same_col() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 0, 0]).unwrap();
    // Col 1 written twice — last value wins.
    assert!(c.multi_set(1, vec![(1, 5), (1, 99)]).unwrap());
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[1], 99);
}

#[test]
fn multi_set_pk_column_can_be_written() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 0, 0]).unwrap();
    // Overwriting col 0 (pk) with same value keeps row consistent.
    assert!(c.multi_set(1, vec![(0, 1), (2, 55)]).unwrap());
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[2], 55);
}

#[test]
fn multi_set_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20, 30]).unwrap();
    c.insert(2, vec![2, 11, 22, 33]).unwrap();
    assert!(c.multi_set(1, vec![(1, 99)]).unwrap());
    let row2 = c.get(2).unwrap().unwrap();
    assert_eq!(row2[1], 11); // untouched
}
