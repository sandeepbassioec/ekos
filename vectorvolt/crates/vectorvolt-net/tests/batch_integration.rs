//! Phase 7: batch-insert and batch-delete integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
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

// ── batch_insert ──────────────────────────────────────────────────────────────

#[test]
fn batch_insert_empty_succeeds() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_insert(vec![]).unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 0);
}

#[test]
fn batch_insert_single_row() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_insert(vec![(1, vec![1, 100])]).unwrap();
    assert_eq!(succeeded, 1);
    assert_eq!(skipped, 0);

    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row, vec![1, 100]);
}

#[test]
fn batch_insert_multiple_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let rows: Vec<(u64, Vec<u64>)> = (1u64..=5).map(|pk| (pk, vec![pk, pk * 10])).collect();
    let (succeeded, skipped) = c.batch_insert(rows).unwrap();
    assert_eq!(succeeded, 5);
    assert_eq!(skipped, 0);

    for pk in 1u64..=5 {
        let row = c.get(pk).unwrap().unwrap();
        assert_eq!(row, vec![pk, pk * 10]);
    }
}

#[test]
fn batch_insert_all_rows_visible_after_insert() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let rows: Vec<(u64, Vec<u64>)> = (10u64..=19).map(|pk| (pk, vec![pk, 0])).collect();
    let (succeeded, _) = c.batch_insert(rows).unwrap();
    assert_eq!(succeeded, 10);

    let scan = c.range_scan(0, 10, 19, 0).unwrap();
    assert_eq!(scan.len(), 10);
}

// ── batch_delete ──────────────────────────────────────────────────────────────

#[test]
fn batch_delete_empty_succeeds() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_delete(vec![]).unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 0);
}

#[test]
fn batch_delete_existing_rows() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.batch_insert(vec![(1, vec![1, 1]), (2, vec![2, 2]), (3, vec![3, 3])])
        .unwrap();

    let (succeeded, skipped) = c.batch_delete(vec![1, 2, 3]).unwrap();
    assert_eq!(succeeded, 3);
    assert_eq!(skipped, 0);

    assert!(c.get(1).unwrap().is_none());
    assert!(c.get(2).unwrap().is_none());
    assert!(c.get(3).unwrap().is_none());
}

#[test]
fn batch_delete_missing_keys_counted_as_skipped() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Insert only pk=1; delete pk=1 and pk=99 (not present).
    c.insert(1, vec![1, 1]).unwrap();

    let (succeeded, skipped) = c.batch_delete(vec![1, 99]).unwrap();
    assert_eq!(succeeded, 1);
    assert_eq!(skipped, 1);
}

#[test]
fn batch_delete_all_missing() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let (succeeded, skipped) = c.batch_delete(vec![100, 200, 300]).unwrap();
    assert_eq!(succeeded, 0);
    assert_eq!(skipped, 3);
}

// ── round-trip: batch-insert then batch-delete ────────────────────────────────

#[test]
fn batch_insert_then_batch_delete_round_trip() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();

    let rows: Vec<(u64, Vec<u64>)> = (1u64..=10).map(|pk| (pk, vec![pk, pk * 100])).collect();
    let (ins, _) = c.batch_insert(rows).unwrap();
    assert_eq!(ins, 10);

    // Delete even pks.
    let even: Vec<u64> = (1u64..=10).filter(|pk| pk % 2 == 0).collect();
    let (del, _) = c.batch_delete(even).unwrap();
    assert_eq!(del, 5);

    // Odd pks still exist; even are gone.
    for pk in 1u64..=10 {
        if pk % 2 == 1 {
            assert!(c.get(pk).unwrap().is_some(), "pk={pk} should still exist");
        } else {
            assert!(c.get(pk).unwrap().is_none(), "pk={pk} should be deleted");
        }
    }
}
