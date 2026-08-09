//! Phase 72: CountIf integration tests.

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
fn count_if_empty_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.count_if(1, 1, 0, 100).unwrap(), 0);
}

#[test]
fn count_if_no_match_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 2]).unwrap();
    c.insert(2, vec![2, 3]).unwrap();
    assert_eq!(c.count_if(1, 99, 0, 100).unwrap(), 0);
}

#[test]
fn count_if_counts_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.insert(2, vec![2, 1]).unwrap();
    c.insert(3, vec![3, 2]).unwrap();
    assert_eq!(c.count_if(1, 1, 0, 100).unwrap(), 2);
}

#[test]
fn count_if_all_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=4 {
        c.insert(i, vec![i, 7]).unwrap();
    }
    assert_eq!(c.count_if(1, 7, 0, 100).unwrap(), 4);
}

#[test]
fn count_if_respects_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap(); // outside [10,100]
    c.insert(2, vec![2, 1]).unwrap();
    c.insert(3, vec![3, 50]).unwrap(); // inside [10,100] but different value
    // count_if on col=1 (status), value=1, scan range [10,100] (scan col=1)
    // rows with status=50 and 1 are in scan range [10,100]... wait
    // actually scan is on col=1 (status), lo=10 hi=100
    // pk=1 has status=1 which is outside [10,100] scan
    // pk=2 has status=1 outside [10,100]
    // pk=3 has status=50 inside [10,100]
    assert_eq!(c.count_if(1, 50, 10, 100).unwrap(), 1);
}

#[test]
fn count_if_single_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    c.insert(2, vec![2, 43]).unwrap();
    assert_eq!(c.count_if(1, 42, 0, 100).unwrap(), 1);
}
