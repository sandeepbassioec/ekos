/// Phase 4 WAL recovery integration tests.
///
/// Each test simulates a crash by dropping the engine without flushing, then
/// re-opening the same data directory and verifying that:
///   - unflushed inserts survive (Phase 4)
///   - unflushed deletes survive (already worked; regression coverage)
///   - interleaved mutations converge correctly
///   - flushed rows are not duplicated
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open(dir: &TempDir) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "age".into(), "score".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

fn row(pk: u64, age: u64, score: u64) -> Vec<u64> {
    vec![pk, age, score]
}

// ── unflushed inserts ─────────────────────────────────────────────────────────

#[test]
fn unflushed_insert_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, row(1, 30, 100)).unwrap();
        // drop without flush — simulates crash
    }

    let e = open(&dir);
    let row = e.get(1).unwrap();
    assert!(row.is_some(), "pk=1 must survive restart");
    assert_eq!(row.unwrap().values, vec![1, 30, 100]);
}

#[test]
fn multiple_unflushed_inserts_survive_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        for pk in 1u64..=5 {
            e.insert(pk, row(pk, pk * 10, pk * 100)).unwrap();
        }
        // drop without flush
    }

    let e = open(&dir);
    for pk in 1u64..=5 {
        let r = e.get(pk).unwrap();
        assert!(r.is_some(), "pk={pk} must survive restart");
        assert_eq!(r.unwrap().values, vec![pk, pk * 10, pk * 100]);
    }
}

// ── unflushed deletes ─────────────────────────────────────────────────────────

#[test]
fn unflushed_delete_of_flushed_row_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(10, row(10, 50, 200)).unwrap();
        e.flush().unwrap();
        e.delete(10).unwrap();
        // drop without flushing the delete — the delete is in the WAL but pk=10
        // is in the SST
    }

    let e = open(&dir);
    let r = e.get(10).unwrap();
    assert!(r.is_none(), "deleted pk=10 must stay deleted after restart");
}

#[test]
fn unflushed_insert_then_delete_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(7, row(7, 20, 30)).unwrap();
        e.delete(7).unwrap();
        // both in WAL, neither flushed
    }

    let e = open(&dir);
    let r = e.get(7).unwrap();
    assert!(
        r.is_none(),
        "insert+delete without flush: row must be absent"
    );
}

// ── mixed flushed + unflushed ─────────────────────────────────────────────────

#[test]
fn flushed_rows_not_doubled_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, row(1, 10, 100)).unwrap();
        e.flush().unwrap();
        // no unflushed inserts
    }

    let e = open(&dir);
    // scan must return exactly one copy of pk=1
    let rows = e.scan(0, 0, u64::MAX).unwrap();
    assert_eq!(
        rows.len(),
        1,
        "flushed row must not be duplicated: {rows:?}"
    );
    assert_eq!(rows[0].pk, 1);
}

#[test]
fn flushed_and_unflushed_rows_coexist_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, row(1, 10, 100)).unwrap();
        e.flush().unwrap();
        e.insert(2, row(2, 20, 200)).unwrap();
        // pk=1 in SST, pk=2 only in WAL
    }

    let e = open(&dir);
    let r1 = e.get(1).unwrap();
    let r2 = e.get(2).unwrap();
    assert!(r1.is_some(), "flushed pk=1 must be found");
    assert!(r2.is_some(), "unflushed pk=2 must survive restart");
    assert_eq!(r1.unwrap().values, vec![1, 10, 100]);
    assert_eq!(r2.unwrap().values, vec![2, 20, 200]);
}

// ── secondary index & fk filter after recovery ───────────────────────────────

#[test]
fn secondary_index_rebuilt_after_wal_recovery() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, row(1, 55, 0)).unwrap();
        e.insert(2, row(2, 55, 0)).unwrap();
        e.insert(3, row(3, 99, 0)).unwrap();
        // none flushed
    }

    let e = open(&dir);
    let rows = e.secondary_scan(1, 55).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(
        pks,
        vec![1, 2],
        "secondary index must be rebuilt from replayed WAL inserts"
    );
}

#[test]
fn fk_filter_correct_after_wal_recovery() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(42, row(42, 1, 2)).unwrap();
        // not flushed
    }

    let e = open(&dir);
    assert!(
        e.fk_exists(42),
        "fk_filter must contain pk=42 after recovery"
    );
    assert!(!e.fk_exists(99999), "fk_filter must not have pk=99999");
}
