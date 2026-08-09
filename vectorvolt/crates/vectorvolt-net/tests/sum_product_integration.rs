//! Phase 84: SumProduct integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "qty".into(), "price".into()]
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
fn sum_product_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let result = c.sum_product(1, 2, 100, 200).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn sum_product_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 3, 4]).unwrap();
    // scan col_a=1 in [3,3], sum_product = 3*4 = 12
    let result = c.sum_product(1, 2, 3, 3).unwrap();
    assert_eq!(result, 12);
}

#[test]
fn sum_product_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 2, 5]).unwrap();
    c.insert(2, vec![2, 3, 4]).unwrap();
    c.insert(3, vec![3, 6, 1]).unwrap();
    // scan col_a in [2,6]: 2*5 + 3*4 + 6*1 = 10+12+6 = 28
    let result = c.sum_product(1, 2, 2, 6).unwrap();
    assert_eq!(result, 28);
}

#[test]
fn sum_product_partial_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 2, 5]).unwrap();
    c.insert(2, vec![2, 10, 3]).unwrap();
    // scan col_a in [2,2]: only pk=1 qualifies; 2*5 = 10
    let result = c.sum_product(1, 2, 2, 2).unwrap();
    assert_eq!(result, 10);
}

#[test]
fn sum_product_zero_col_b() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 0]).unwrap();
    c.insert(2, vec![2, 5, 0]).unwrap();
    let result = c.sum_product(1, 2, 5, 5).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn sum_product_zero_col_a() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 99]).unwrap();
    // scan col_a in [0,0]: 0*99 = 0
    let result = c.sum_product(1, 2, 0, 0).unwrap();
    assert_eq!(result, 0);
}
