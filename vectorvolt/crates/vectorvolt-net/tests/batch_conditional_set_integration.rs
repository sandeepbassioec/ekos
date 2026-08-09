//! Phase 81: BatchConditionalSet integration tests.

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
fn batch_conditional_set_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.batch_conditional_set(1, 10, 99, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn batch_conditional_set_matches_expected() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    // expected=10 → all match → all updated
    let affected = c.batch_conditional_set(1, 10, 99, 10, 10).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 99);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 99);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 99);
}

#[test]
fn batch_conditional_set_no_match_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    // expected=99 does not match 10 → no update
    c.batch_conditional_set(1, 99, 42, 10, 10).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
}

#[test]
fn batch_conditional_set_partial_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    c.insert(3, vec![3, 5]).unwrap();
    // scan [5,5]; only rows where col=5 match expected=5
    // set new_value=77
    let affected = c.batch_conditional_set(1, 5, 77, 5, 5).unwrap();
    assert_eq!(affected, 3);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
}

#[test]
fn batch_conditional_set_out_of_range_untouched() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 50]).unwrap();
    // scan [50,50]; pk=1 col=10 not in scan range
    c.batch_conditional_set(1, 50, 0, 50, 50).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 0);
}

#[test]
fn batch_conditional_set_zero_new_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 25]).unwrap();
    c.batch_conditional_set(1, 25, 0, 25, 25).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 0);
}
