//! Phase 83: WindowPct integration tests.

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
fn window_pct_missing_pk_returns_none() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert!(c.window_pct(99, 1, 1, 100).unwrap().is_none());
}

#[test]
fn window_pct_sole_row_is_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 50]).unwrap();
    // 0 rows below, 1 total → 0 * 100 / 1 = 0
    let pct = c.window_pct(1, 1, 50, 50).unwrap().unwrap();
    assert_eq!(pct, 0);
}

#[test]
fn window_pct_last_of_four() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    c.insert(4, vec![4, 40]).unwrap();
    // pk=4 score=40; 3 below, 4 total → 3*100/4 = 75
    let pct = c.window_pct(4, 1, 10, 40).unwrap().unwrap();
    assert_eq!(pct, 75);
}

#[test]
fn window_pct_first_of_four() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    c.insert(4, vec![4, 40]).unwrap();
    // pk=1 score=10; 0 below → 0
    let pct = c.window_pct(1, 1, 10, 40).unwrap().unwrap();
    assert_eq!(pct, 0);
}

#[test]
fn window_pct_second_of_four() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();
    c.insert(4, vec![4, 40]).unwrap();
    // pk=2 score=20; 1 below, 4 total → 1*100/4 = 25
    let pct = c.window_pct(2, 1, 10, 40).unwrap().unwrap();
    assert_eq!(pct, 25);
}

#[test]
fn window_pct_tied_values_all_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    // all tied; 0 below any → pct = 0
    let pct = c.window_pct(2, 1, 10, 10).unwrap().unwrap();
    assert_eq!(pct, 0);
}
