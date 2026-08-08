//! Phase 32: BatchSetField integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "status".into(), "val".into()]
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
fn batch_set_field_empty_pks_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let n = c.batch_set_field(1, 99, vec![]).unwrap();
    assert_eq!(n, 0);
}

#[test]
fn batch_set_field_updates_all_present_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=4 {
        c.insert(pk, vec![pk, 0, pk * 10]).unwrap();
    }
    let n = c.batch_set_field(1, 99, vec![1, 2, 3, 4]).unwrap();
    assert_eq!(n, 4);
    for pk in 1u64..=4 {
        assert_eq!(c.get(pk).unwrap().unwrap()[1], 99);
    }
}

#[test]
fn batch_set_field_skips_missing_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 10]).unwrap();
    // pk=2 and pk=3 don't exist
    let n = c.batch_set_field(1, 7, vec![1, 2, 3]).unwrap();
    assert_eq!(n, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 7);
}

#[test]
fn batch_set_field_does_not_affect_other_columns() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 42]).unwrap();
    c.batch_set_field(1, 99, vec![1]).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row[2], 42); // col 2 untouched
}

#[test]
fn batch_set_field_partial_update() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5, 0]).unwrap();
    c.insert(2, vec![2, 5, 0]).unwrap();
    c.insert(3, vec![3, 5, 0]).unwrap();
    // Only update rows 1 and 3
    let n = c.batch_set_field(1, 100, vec![1, 3]).unwrap();
    assert_eq!(n, 2);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
    assert_eq!(c.get(2).unwrap().unwrap()[1], 5); // unchanged
    assert_eq!(c.get(3).unwrap().unwrap()[1], 100);
}

#[test]
fn batch_set_field_idempotent_on_second_call() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0, 0]).unwrap();
    c.batch_set_field(1, 42, vec![1]).unwrap();
    let n = c.batch_set_field(1, 42, vec![1]).unwrap();
    assert_eq!(n, 1);
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
}

#[test]
fn batch_set_field_large_batch() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=20 {
        c.insert(pk, vec![pk, 0, 0]).unwrap();
    }
    let pks: Vec<u64> = (1u64..=20).collect();
    let n = c.batch_set_field(1, 255, pks).unwrap();
    assert_eq!(n, 20);
    for pk in 1u64..=20 {
        assert_eq!(c.get(pk).unwrap().unwrap()[1], 255);
    }
}
