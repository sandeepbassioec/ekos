//! Phase 89: FieldHistogram integration tests.

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
fn field_histogram_empty_range() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let rows = c.field_histogram(1, 10, 100, 200).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn field_histogram_single_bucket() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    c.insert(2, vec![2, 8]).unwrap();
    c.insert(3, vec![3, 9]).unwrap();
    // bucket_size=10: all values 5,8,9 in bucket starting at 0
    let rows = c.field_histogram(1, 10, 5, 9).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 0); // bucket_start = 0
    assert_eq!(rows[0][1], 3); // count = 3
}

#[test]
fn field_histogram_multiple_buckets() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 5]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    c.insert(4, vec![4, 15]).unwrap();
    // bucket_size=10: bucket 0=[0,9] → 2 rows, bucket 10=[10,19] → 2 rows
    let rows = c.field_histogram(1, 10, 0, 15).unwrap();
    assert_eq!(rows.len(), 2);
    // sorted by bucket start
    let b0 = rows.iter().find(|r| r[0] == 0).unwrap();
    let b10 = rows.iter().find(|r| r[0] == 10).unwrap();
    assert_eq!(b0[1], 2);
    assert_eq!(b10[1], 2);
}

#[test]
fn field_histogram_bucket_size_zero_treated_as_one() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 3]).unwrap();
    c.insert(2, vec![2, 3]).unwrap();
    // bucket_size=0 → treated as 1; each unique value is its own bucket
    let rows = c.field_histogram(1, 0, 3, 3).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], 3);
    assert_eq!(rows[0][1], 2);
}

#[test]
fn field_histogram_each_unique_value_own_bucket() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.insert(2, vec![2, 2]).unwrap();
    c.insert(3, vec![3, 3]).unwrap();
    let rows = c.field_histogram(1, 1, 1, 3).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn field_histogram_counts_per_bucket() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 10]).unwrap();
    c.insert(3, vec![3, 10]).unwrap();
    let rows = c.field_histogram(1, 10, 10, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][1], 3);
}
