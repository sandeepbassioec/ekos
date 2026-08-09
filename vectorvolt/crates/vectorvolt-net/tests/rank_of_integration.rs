//! Phase 43: RankOf integration tests.

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
fn rank_of_absent_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.rank_of(99, 1, 0, u64::MAX).unwrap().is_none());
}

#[test]
fn rank_of_sole_row_is_rank_0() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    assert_eq!(c.rank_of(1, 1, 0, 100).unwrap(), Some(0));
}

#[test]
fn rank_of_highest_in_three_is_rank_2() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.rank_of(3, 1, 0, 100).unwrap(), Some(2));
}

#[test]
fn rank_of_lowest_in_three_is_rank_0() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.rank_of(1, 1, 0, 100).unwrap(), Some(0));
}

#[test]
fn rank_of_middle_is_rank_1() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    assert_eq!(c.rank_of(2, 1, 0, 100).unwrap(), Some(1));
}

#[test]
fn rank_of_outside_scan_range_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap(); // score=5, range [10, 100]
    assert!(c.rank_of(1, 1, 10, 100).unwrap().is_none());
}

#[test]
fn rank_of_does_not_modify_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.rank_of(1, 1, 0, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}
