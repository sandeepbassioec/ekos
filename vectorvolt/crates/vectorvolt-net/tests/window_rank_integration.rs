//! Phase 82: WindowRank integration tests.

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
fn window_rank_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.window_rank(99, 1, 1, 100).unwrap().is_none());
}

#[test]
fn window_rank_sole_row_is_rank_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    let rank = c.window_rank(1, 1, 50, 50).unwrap().unwrap();
    assert_eq!(rank, 1);
}

#[test]
fn window_rank_last_is_highest() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    // pk=3 has score=30; window [10,30]; 2 rows below it → rank 3
    let rank = c.window_rank(3, 1, 10, 30).unwrap().unwrap();
    assert_eq!(rank, 3);
}

#[test]
fn window_rank_first_is_rank_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rank = c.window_rank(1, 1, 10, 30).unwrap().unwrap();
    assert_eq!(rank, 1);
}

#[test]
fn window_rank_middle_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rank = c.window_rank(2, 1, 10, 30).unwrap().unwrap();
    assert_eq!(rank, 2);
}

#[test]
fn window_rank_tied_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    // all tied at 10; 0 rows below any → all rank 1
    let rank = c.window_rank(2, 1, 10, 10).unwrap().unwrap();
    assert_eq!(rank, 1);
}
