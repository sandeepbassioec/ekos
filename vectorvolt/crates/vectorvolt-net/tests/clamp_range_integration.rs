//! Phase 80: ClampRange integration tests.

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
fn clamp_range_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let affected = c.clamp_range(1, 10, 50, 100, 200).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn clamp_range_clamps_below_min() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 3]).unwrap();
    // scan [3,3], clamp to [10, 50]; value 3 < 10 → set to 10
    c.clamp_range(1, 10, 50, 3, 3).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
}

#[test]
fn clamp_range_clamps_above_max() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    // scan [100,100], clamp to [10, 50]; value 100 > 50 → set to 50
    c.clamp_range(1, 10, 50, 100, 100).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 50);
}

#[test]
fn clamp_range_in_bounds_unchanged() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 25]).unwrap();
    // scan [25,25], clamp to [10, 50]; value 25 in bounds → unchanged
    c.clamp_range(1, 10, 50, 25, 25).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 25);
}

#[test]
fn clamp_range_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 30]).unwrap();
    c.insert(3, vec![3, 80]).unwrap();
    // scan [5,80], clamp to [10, 50]
    let affected = c.clamp_range(1, 10, 50, 5, 80).unwrap();
    // only rows that changed are counted (value 30 is already in [10,50])
    assert_eq!(affected, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 10);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 30);
    assert_eq!(c.get(3).unwrap().unwrap()[1], 50);
}

#[test]
fn clamp_range_out_of_scan_range_untouched() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 200]).unwrap();
    // scan [5,50] — value 200 not in scan range; untouched
    c.clamp_range(1, 10, 50, 5, 50).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 200);
}
