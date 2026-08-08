//! Phase 18: ScanKeys integration tests.

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

// ── basic behaviour ───────────────────────────────────────────────────────────

#[test]
fn scan_keys_empty_table_returns_empty() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let pks = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    assert!(pks.is_empty());
}

#[test]
fn scan_keys_returns_inserted_pks() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.insert(2, vec![2, 20]).unwrap();
    c.insert(3, vec![3, 30]).unwrap();

    let mut pks = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 2, 3]);
}

#[test]
fn scan_keys_returns_only_pks_not_values() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(42, vec![42, 999]).unwrap();

    let pks = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    assert_eq!(pks, vec![42]);
    // Confirm it's just the PK, not including the value column.
    assert_eq!(pks.len(), 1);
}

// ── range filter ──────────────────────────────────────────────────────────────

#[test]
fn scan_keys_respects_pk_range() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    let mut pks = c.scan_keys(0, 3, 7, 0).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![3, 4, 5, 6, 7]);
}

// ── limit ─────────────────────────────────────────────────────────────────────

#[test]
fn scan_keys_limit_caps_results() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    let pks = c.scan_keys(0, 0, u64::MAX, 3).unwrap();
    assert_eq!(pks.len(), 3);
}

#[test]
fn scan_keys_zero_limit_means_unlimited() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=20 {
        c.insert(pk, vec![pk, 0]).unwrap();
    }

    let pks = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    assert_eq!(pks.len(), 20);
}

// ── consistency with range_scan ───────────────────────────────────────────────

#[test]
fn scan_keys_pks_match_range_scan_first_column() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=5 {
        c.insert(pk, vec![pk, pk * 100]).unwrap();
    }

    let mut keys_only = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    let mut full_rows = c.range_scan(0, 0, u64::MAX, 0).unwrap();
    keys_only.sort_unstable();
    full_rows.sort_by_key(|r| r[0]);

    let pks_from_rows: Vec<u64> = full_rows.iter().map(|r| r[0]).collect();
    assert_eq!(keys_only, pks_from_rows);
}

// ── single key ───────────────────────────────────────────────────────────────

#[test]
fn scan_keys_single_row_table() {
    let (_tmp, server, addr) = start_server();
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let big_pk: u64 = u64::MAX - 1;
    c.insert(big_pk, vec![big_pk, 0]).unwrap();

    let pks = c.scan_keys(0, 0, u64::MAX, 0).unwrap();
    assert_eq!(pks, vec![big_pk]);
}
