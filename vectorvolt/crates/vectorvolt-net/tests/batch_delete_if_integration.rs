//! Phase 51: BatchDeleteIf integration tests.

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
fn batch_delete_if_empty_pks_returns_zero() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_delete_if(1, 0, vec![]).unwrap(), 0);
}

#[test]
fn batch_delete_if_absent_pks_skipped() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    assert_eq!(c.batch_delete_if(1, 0, vec![1, 2, 3]).unwrap(), 0);
}

#[test]
fn batch_delete_if_deletes_matching_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    c.insert(3, vec![3, 1]).unwrap();
    let count = c.batch_delete_if(1, 0, vec![1, 2, 3]).unwrap();
    assert_eq!(count, 2);
    assert!(c.get(1).unwrap().is_none());
    assert!(c.get(2).unwrap().is_none());
    assert!(c.get(3).unwrap().is_some());
}

#[test]
fn batch_delete_if_mismatch_not_deleted() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 5]).unwrap();
    let count = c.batch_delete_if(1, 99, vec![1]).unwrap();
    assert_eq!(count, 0);
    assert!(c.get(1).unwrap().is_some());
}

#[test]
fn batch_delete_if_all_match() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    for i in 1u64..=5 {
        c.insert(i, vec![i, 7]).unwrap();
    }
    let count = c.batch_delete_if(1, 7, vec![1, 2, 3, 4, 5]).unwrap();
    assert_eq!(count, 5);
    for i in 1u64..=5 {
        assert!(c.get(i).unwrap().is_none());
    }
}

#[test]
fn batch_delete_if_does_not_affect_other_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    c.insert(2, vec![2, 0]).unwrap();
    c.insert(3, vec![3, 0]).unwrap();
    // only delete pk=1
    c.batch_delete_if(1, 0, vec![1]).unwrap();
    assert!(c.get(2).unwrap().is_some());
    assert!(c.get(3).unwrap().is_some());
}

#[test]
fn batch_delete_if_zero_value() {
    let (_tmp, server, addr) = start_server();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 0]).unwrap();
    let count = c.batch_delete_if(1, 0, vec![1]).unwrap();
    assert_eq!(count, 1);
    assert!(c.get(1).unwrap().is_none());
}
