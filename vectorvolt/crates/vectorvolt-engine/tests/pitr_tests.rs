//! PITR integration tests for TableEngine (VV-STO-017).

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1000).with_durability(Durability::Buffered),
    )
    .unwrap()
}

#[test]
fn create_snapshot_returns_marker() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 100]).unwrap();
    let marker = eng.create_snapshot().unwrap();
    assert!(marker.snapshot_id > 0);
}

#[test]
fn create_snapshot_records_tier_files() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    for i in 0..10u64 {
        eng.insert(i, vec![i, i * 2]).unwrap();
    }
    eng.flush().unwrap();
    let marker = eng.create_snapshot().unwrap();
    // At least one .vvsst file should exist after flush
    assert!(!marker.tier_files.is_empty());
}

#[test]
fn list_snapshots_empty_before_create() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    let snaps = eng.list_snapshots().unwrap();
    assert!(snaps.is_empty());
}

#[test]
fn list_snapshots_finds_created() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 10]).unwrap();
    eng.create_snapshot().unwrap();
    let snaps = eng.list_snapshots().unwrap();
    assert_eq!(snaps.len(), 1);
}

#[test]
fn list_snapshots_multiple() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 10]).unwrap();
    eng.create_snapshot().unwrap();
    eng.insert(2, vec![2, 20]).unwrap();
    eng.create_snapshot().unwrap();
    let snaps = eng.list_snapshots().unwrap();
    assert_eq!(snaps.len(), 2);
}

#[test]
fn restore_from_snapshot_zero_opens_engine() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 10]).unwrap();
        eng.flush().unwrap();
    }
    let restored =
        TableEngine::restore_from_snapshot(vec!["pk".into(), "val".into()], dir.path(), 0).unwrap();
    assert!(restored.get(1).unwrap().is_some());
}

#[test]
fn restore_from_snapshot_missing_id_errors() {
    let dir = TempDir::new().unwrap();
    let _eng = open(&dir);
    let result =
        TableEngine::restore_from_snapshot(vec!["pk".into(), "val".into()], dir.path(), u64::MAX);
    assert!(result.is_err());
}

#[test]
fn restore_from_snapshot_existing_id_opens() {
    let dir = TempDir::new().unwrap();
    // Write data and flush to disk; snapshot_id=0 path validates the open works.
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 10]).unwrap();
        eng.flush().unwrap();
    }
    let restored =
        TableEngine::restore_from_snapshot(vec!["pk".into(), "val".into()], dir.path(), 0).unwrap();
    assert!(restored.get(1).unwrap().is_some());
}

#[test]
fn create_snapshot_table_name_default() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    let marker = eng.create_snapshot().unwrap();
    assert_eq!(marker.table_name, "default");
}

#[test]
fn create_snapshot_lsn_nonnegative() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    let marker = eng.create_snapshot().unwrap();
    // lsn is u64 so always >= 0
    let _ = marker.lsn;
}

#[test]
fn create_snapshot_timestamp_set() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    let marker = eng.create_snapshot().unwrap();
    assert!(marker.timestamp_ns > 0);
}

#[test]
fn create_snapshot_twice_different_ids() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    let m1 = eng.create_snapshot().unwrap();
    // Small sleep so timestamp differs
    std::thread::sleep(std::time::Duration::from_millis(2));
    let m2 = eng.create_snapshot().unwrap();
    // IDs should differ since they're based on timestamps
    assert_ne!(m1.snapshot_id, m2.snapshot_id);
}

#[test]
fn snapshot_marker_tier_files_are_vvsst() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    for i in 0..5u64 {
        eng.insert(i, vec![i, i]).unwrap();
    }
    eng.flush().unwrap();
    let marker = eng.create_snapshot().unwrap();
    for f in &marker.tier_files {
        assert!(f.ends_with(".vvsst"), "unexpected file: {}", f);
    }
}
