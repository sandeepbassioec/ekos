//! Phase 8: server-side transaction integration tests.

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

// ── begin / commit ────────────────────────────────────────────────────────────

#[test]
fn begin_returns_txn_id() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    assert!(id > 0);
}

#[test]
fn commit_empty_txn_succeeds() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    let affected = c.commit_txn(id).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn inserted_rows_visible_after_commit() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    // Rows buffered during transaction — not yet applied.
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    // Before commit rows may or may not be visible (they are buffered);
    // after commit they must be.
    let affected = c.commit_txn(id).unwrap();
    assert_eq!(affected, 2);

    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 100]);
    assert_eq!(c.get(2).unwrap().unwrap(), vec![2, 200]);
}

#[test]
fn txn_id_increments_across_transactions() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id1 = c.begin_txn().unwrap();
    c.commit_txn(id1).unwrap();

    let id2 = c.begin_txn().unwrap();
    c.commit_txn(id2).unwrap();

    assert_ne!(id1, id2);
}

// ── rollback ──────────────────────────────────────────────────────────────────

#[test]
fn rollback_discards_buffered_ops() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    c.insert(42, vec![42, 9999]).unwrap();
    c.rollback_txn(id).unwrap();

    // Row must not exist because the insert was rolled back.
    assert!(c.get(42).unwrap().is_none());
}

#[test]
fn rollback_empty_txn_succeeds() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    c.rollback_txn(id).unwrap();
}

// ── error paths ───────────────────────────────────────────────────────────────

#[test]
fn begin_while_active_txn_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let _id = c.begin_txn().unwrap();
    // Second begin on same connection should fail.
    assert!(c.begin_txn().is_err());
}

#[test]
fn commit_without_active_txn_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert!(c.commit_txn(1).is_err());
}

#[test]
fn rollback_without_active_txn_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert!(c.rollback_txn(1).is_err());
}

#[test]
fn commit_with_wrong_txn_id_returns_error() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let id = c.begin_txn().unwrap();
    assert!(c.commit_txn(id + 1).is_err());
    // Transaction still active after mismatch — can still rollback.
    c.rollback_txn(id).unwrap();
}

// ── delete inside txn ─────────────────────────────────────────────────────────

#[test]
fn delete_inside_txn_applied_on_commit() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    // Insert a row outside any transaction.
    c.insert(10, vec![10, 1000]).unwrap();
    assert!(c.get(10).unwrap().is_some());

    // Delete it inside a transaction.
    let id = c.begin_txn().unwrap();
    c.delete(10).unwrap();
    c.commit_txn(id).unwrap();

    assert!(c.get(10).unwrap().is_none());
}

#[test]
fn delete_inside_txn_not_applied_on_rollback() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(11, vec![11, 1100]).unwrap();

    let id = c.begin_txn().unwrap();
    c.delete(11).unwrap();
    c.rollback_txn(id).unwrap();

    // Row must still exist.
    assert!(c.get(11).unwrap().is_some());
}

// ── two connections are independent ──────────────────────────────────────────

#[test]
fn two_connections_have_independent_transactions() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c1 = Client::connect(addr).unwrap();
    let mut c2 = Client::connect(addr).unwrap();

    // c1 starts a txn and inserts.
    let id1 = c1.begin_txn().unwrap();
    c1.insert(100, vec![100, 1]).unwrap();

    // c2 starts its own txn and inserts different row.
    let id2 = c2.begin_txn().unwrap();
    c2.insert(200, vec![200, 2]).unwrap();

    // Commit both.
    c1.commit_txn(id1).unwrap();
    c2.commit_txn(id2).unwrap();

    // Both rows visible on a fresh connection.
    let mut c3 = Client::connect(addr).unwrap();
    assert!(c3.get(100).unwrap().is_some());
    assert!(c3.get(200).unwrap().is_some());
}
