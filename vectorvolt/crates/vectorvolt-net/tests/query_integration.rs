/// Wire-level integration tests for `SecondaryLookup` and `FkExists` opcodes.
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_net::{Client, Server, ServerConfig};

struct Harness {
    addr: std::net::SocketAddr,
    shutdown: vectorvolt_net::server::ShutdownHandle,
    _dir: TempDir,
}

impl Harness {
    fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let engine = Arc::new(
            TableEngine::open_with(
                vec!["pk".into(), "age".into(), "score".into()],
                dir.path(),
                EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
            )
            .unwrap(),
        );
        let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
        let server = Server::bind(engine, cfg).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        std::thread::spawn(move || {
            let _ = server.serve();
        });
        Harness {
            addr,
            shutdown,
            _dir: dir,
        }
    }

    fn client(&self) -> Client {
        let c = Client::connect(self.addr).unwrap();
        c.set_timeout(Some(Duration::from_secs(5))).unwrap();
        c
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

// ── secondary_scan ────────────────────────────────────────────────────────────

#[test]
fn secondary_scan_returns_matching_rows() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(1, vec![1, 30, 100]).unwrap();
    c.insert(2, vec![2, 40, 200]).unwrap();
    c.insert(3, vec![3, 30, 300]).unwrap();

    let rows = c.secondary_scan(1, 30).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 3]);
}

#[test]
fn secondary_scan_returns_empty_for_no_match() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(1, vec![1, 30, 0]).unwrap();

    let rows = c.secondary_scan(1, 999).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn secondary_scan_returns_full_row_values() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(7, vec![7, 42, 88]).unwrap();

    let rows = c.secondary_scan(1, 42).unwrap();
    assert_eq!(rows, vec![vec![7, 42, 88]]);
}

#[test]
fn secondary_scan_col2_works() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(10, vec![10, 5, 77]).unwrap();
    c.insert(20, vec![20, 6, 77]).unwrap();
    c.insert(30, vec![30, 7, 88]).unwrap();

    let rows = c.secondary_scan(2, 77).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![10, 20]);
}

#[test]
fn secondary_scan_after_delete_excludes_deleted_row() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(1, vec![1, 55, 0]).unwrap();
    c.insert(2, vec![2, 55, 0]).unwrap();
    c.delete(1).unwrap();

    let rows = c.secondary_scan(1, 55).unwrap();
    let pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    assert!(!pks.contains(&1));
    assert!(pks.contains(&2));
}

#[test]
fn secondary_scan_on_empty_table_returns_empty() {
    let h = Harness::start();
    let mut c = h.client();
    let rows = c.secondary_scan(1, 42).unwrap();
    assert!(rows.is_empty());
}

// ── fk_exists ─────────────────────────────────────────────────────────────────

#[test]
fn fk_exists_true_for_inserted_pk() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(99, vec![99, 1, 2]).unwrap();

    assert!(c.fk_exists(99).unwrap());
}

#[test]
fn fk_exists_false_for_never_inserted_pk() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(1, vec![1, 1, 1]).unwrap();

    assert!(!c.fk_exists(99999).unwrap());
}

#[test]
fn fk_exists_on_empty_table_returns_false() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(!c.fk_exists(1).unwrap());
}

#[test]
fn fk_exists_false_for_deleted_pk() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(44, vec![44, 1, 2]).unwrap();
    c.delete(44).unwrap();

    // The filter may still hold the key, but positives are confirmed against
    // the pk index (PP-39), so a deleted pk is reported absent.
    assert!(!c.fk_exists(44).unwrap());
}

#[test]
fn fk_exists_no_false_negatives_for_batch() {
    let h = Harness::start();
    let mut c = h.client();

    for pk in 1u64..=50 {
        c.insert(pk, vec![pk, pk, pk]).unwrap();
    }

    let false_negatives = (1u64..=50).filter(|&pk| !c.fk_exists(pk).unwrap()).count();
    assert_eq!(false_negatives, 0);
}

// ── both together ─────────────────────────────────────────────────────────────

#[test]
fn secondary_scan_and_fk_exists_on_same_connection() {
    let h = Harness::start();
    let mut c = h.client();

    c.insert(5, vec![5, 10, 20]).unwrap();
    c.insert(6, vec![6, 10, 30]).unwrap();

    let rows = c.secondary_scan(1, 10).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(c.fk_exists(5).unwrap());
    assert!(!c.fk_exists(999).unwrap());
}
