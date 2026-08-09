//! Phase 86: DenseRankWin integration tests.

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
fn dense_rank_win_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.dense_rank_win(99, 1, 1, 100).unwrap().is_none());
}

#[test]
fn dense_rank_win_sole_row_is_rank_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    let rank = c.dense_rank_win(1, 1, 50, 50).unwrap().unwrap();
    assert_eq!(rank, 1);
}

#[test]
fn dense_rank_win_distinct_values() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    // pk=3 score=30; 2 distinct values below (10, 20) → dense rank 3
    let rank = c.dense_rank_win(3, 1, 10, 30).unwrap().unwrap();
    assert_eq!(rank, 3);
}

#[test]
fn dense_rank_win_tied_values_same_rank() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    // pk=1,2 tied at 10; 0 distinct values below → both rank 1
    let rank1 = c.dense_rank_win(1, 1, 10, 20).unwrap().unwrap();
    let rank2 = c.dense_rank_win(2, 1, 10, 20).unwrap().unwrap();
    assert_eq!(rank1, 1);
    assert_eq!(rank2, 1);
}

#[test]
fn dense_rank_win_after_tied_group() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 20]).unwrap();
    // pk=3 score=20; 1 distinct value below (10) → dense rank 2
    let rank = c.dense_rank_win(3, 1, 10, 20).unwrap().unwrap();
    assert_eq!(rank, 2);
}

#[test]
fn dense_rank_win_first_is_rank_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    let rank = c.dense_rank_win(1, 1, 10, 30).unwrap().unwrap();
    assert_eq!(rank, 1);
}
