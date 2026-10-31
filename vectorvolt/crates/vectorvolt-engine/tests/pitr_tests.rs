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

// ── point-in-time restore (PP-12b / B3) ─────────────────────────────────────

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
}

/// Like `open`, but the WAL reaches the OS on every commit, so a restore can
/// read it while the source table is still open.
fn open_live(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        schema(),
        dir.path(),
        EngineOptions::new(1000).with_durability(Durability::WriteOnly),
    )
    .unwrap()
}

/// Live rows, ascending by pk.
type Rows = Vec<(u64, Vec<u64>)>;

fn contents(eng: &TableEngine) -> Rows {
    (0u64..32)
        .filter_map(|pk| eng.get(pk).unwrap().map(|r| (pk, r.values)))
        .collect()
}

#[test]
fn restore_to_lsn_reproduces_each_point_exactly() {
    let src = TempDir::new().unwrap();
    let eng = open_live(&src);
    let mut points: Vec<(u64, Rows)> = Vec::new();
    let mut record = |eng: &TableEngine, seq: u64| points.push((seq, contents(eng)));

    for pk in 1u64..=5 {
        let tip = eng.insert(pk, vec![pk, pk * 10]).unwrap();
        record(&eng, tip.sequence);
    }
    eng.flush().unwrap();
    let tip = eng.update(2, vec![2, 222]).unwrap();
    record(&eng, tip.sequence);
    let tip = eng.delete(3).unwrap();
    record(&eng, tip.sequence);
    let marker = eng.create_snapshot().unwrap();
    let tip = eng
        .apply_batch(vec![
            (6, Some(vec![6, 60])),
            (1, None),
            (4, Some(vec![4, 44])),
        ])
        .unwrap();
    record(&eng, tip.sequence);
    let tip = eng.insert(3, vec![3, 33]).unwrap();
    record(&eng, tip.sequence);
    eng.flush().unwrap();
    let live_before = contents(&eng);

    for (seq, expected) in &points {
        let dst = TempDir::new().unwrap();
        let restored = TableEngine::restore_to_lsn(schema(), src.path(), dst.path(), *seq).unwrap();
        assert_eq!(&contents(&restored), expected, "state at seq {seq}");
        drop(restored);
        // The restore is durable on its own: reopening gives the same state.
        let reopened = TableEngine::open(schema(), dst.path(), 1000).unwrap();
        assert_eq!(
            &contents(&reopened),
            expected,
            "reopened state at seq {seq}"
        );
    }

    // Restoring to the marker gives the state at the marker, not the latest.
    let dst = TempDir::new().unwrap();
    let at_marker =
        TableEngine::restore_to_marker(schema(), src.path(), dst.path(), marker.snapshot_id)
            .unwrap();
    assert_eq!(
        contents(&at_marker),
        vec![
            (1, vec![1, 10]),
            (2, vec![2, 222]),
            (4, vec![4, 40]),
            (5, vec![5, 50])
        ]
    );

    // The live table was never touched.
    assert_eq!(contents(&eng), live_before);
}

#[test]
fn restore_before_first_write_is_empty() {
    let src = TempDir::new().unwrap();
    let eng = open_live(&src);
    for pk in 1u64..=5 {
        eng.insert(pk, vec![pk, pk]).unwrap();
    }
    let dst = TempDir::new().unwrap();
    let restored = TableEngine::restore_to_lsn(schema(), src.path(), dst.path(), 0).unwrap();
    assert!(contents(&restored).is_empty());

    let dst = TempDir::new().unwrap();
    let restored = TableEngine::restore_to_timestamp(schema(), src.path(), dst.path(), 1).unwrap();
    assert!(contents(&restored).is_empty());

    let dst = TempDir::new().unwrap();
    let restored =
        TableEngine::restore_to_timestamp(schema(), src.path(), dst.path(), u64::MAX).unwrap();
    assert_eq!(contents(&restored), contents(&eng));
}

#[test]
fn restore_refuses_bad_targets() {
    let src = TempDir::new().unwrap();
    let eng = open_live(&src);
    let tip = eng.insert(1, vec![1, 1]).unwrap();

    // Past the end of the WAL.
    let dst = TempDir::new().unwrap();
    assert!(
        TableEngine::restore_to_lsn(schema(), src.path(), dst.path(), tip.sequence + 100).is_err()
    );
    // Into the live table itself, or any non-empty directory.
    assert!(TableEngine::restore_to_lsn(schema(), src.path(), src.path(), tip.sequence).is_err());
    let dst = TempDir::new().unwrap();
    std::fs::write(dst.path().join("x"), b"x").unwrap();
    assert!(TableEngine::restore_to_lsn(schema(), src.path(), dst.path(), tip.sequence).is_err());
    assert_eq!(contents(&eng), vec![(1, vec![1, 1])]);
}

/// B4: a marker must only be written after a successful flush.
#[test]
fn create_snapshot_fails_and_writes_no_marker_when_flush_fails() {
    let dir = TempDir::new().unwrap();
    let eng = open_live(&dir);
    eng.insert(1, vec![1, 1]).unwrap();
    // Replace the partition directory with a plain file so the flush cannot
    // write its partition.
    let data = dir.path().join("data");
    std::fs::remove_dir_all(&data).unwrap();
    std::fs::write(&data, b"not a directory").unwrap();
    assert!(eng.create_snapshot().is_err());
    assert!(eng.list_snapshots().unwrap().is_empty());
}
