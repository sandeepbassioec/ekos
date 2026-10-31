/// Regression tests for the path-to-production P0 recovery bugs (PP-01..PP-13).
///
/// Each test reproduces a way the engine used to lose or corrupt data across a
/// restart. "Restart" is dropping the engine without a flush and reopening the
/// same directory, which is what a process crash looks like to the files.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
    )
    .unwrap()
}

fn values(e: &TableEngine, pk: u64) -> Option<Vec<u64>> {
    e.get(pk).unwrap().map(|r| r.values)
}

// ── PP-01: a snapshot must not discard writes made after it ───────────────────

#[test]
fn writes_after_a_snapshot_survive_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, vec![1, 10, 100]).unwrap();
        e.take_snapshot().unwrap();
        e.insert(2, vec![2, 20, 200]).unwrap();
        e.delete(1).unwrap();
    }
    let e = open(&dir);
    assert_eq!(
        values(&e, 2),
        Some(vec![2, 20, 200]),
        "post-snapshot insert lost"
    );
    assert_eq!(values(&e, 1), None, "post-snapshot delete lost");
}

#[test]
fn writes_after_a_snapshot_survive_two_restarts() {
    // The old code truncated the WAL on *every* open while a marker existed,
    // so a second restart lost what the first one had kept.
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, vec![1, 1, 1]).unwrap();
        e.take_snapshot().unwrap();
    }
    {
        let e = open(&dir);
        e.insert(2, vec![2, 2, 2]).unwrap();
    }
    drop(open(&dir));
    let e = open(&dir);
    assert_eq!(values(&e, 1), Some(vec![1, 1, 1]));
    assert_eq!(values(&e, 2), Some(vec![2, 2, 2]));
}

#[test]
fn snapshot_still_skips_frames_it_covers() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        for pk in 1..=50u64 {
            e.insert(pk, vec![pk, pk, pk]).unwrap();
        }
        e.take_snapshot().unwrap();
    }
    let e = open(&dir);
    assert_eq!(e.stats().rows_buffered, 0, "covered frames were replayed");
    assert_eq!(values(&e, 50), Some(vec![50, 50, 50]));
}

// ── PP-08: an overwrite of a flushed row must survive restart ─────────────────

#[test]
fn update_of_a_flushed_row_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(7, vec![7, 1, 1]).unwrap();
        e.flush().unwrap();
        // All four overwrite paths, none followed by a flush.
        e.update_field(7, 1, 2).unwrap();
        assert!(e.compare_and_swap(7, &[7, 2, 1], vec![7, 2, 3]).unwrap());
        e.upsert(7, vec![7, 4, 3]).unwrap();
        e.update(7, vec![7, 5, 5]).unwrap();
    }
    let e = open(&dir);
    assert_eq!(values(&e, 7), Some(vec![7, 5, 5]), "overwrite rolled back");
}

#[test]
fn cas_of_a_flushed_row_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(9, vec![9, 0, 0]).unwrap();
        e.flush().unwrap();
        assert!(e.compare_and_swap(9, &[9, 0, 0], vec![9, 1, 1]).unwrap());
    }
    let e = open(&dir);
    assert_eq!(values(&e, 9), Some(vec![9, 1, 1]));
}

#[test]
fn delete_then_reinsert_of_a_flushed_row_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(3, vec![3, 1, 1]).unwrap();
        e.flush().unwrap();
        e.delete(3).unwrap();
        e.insert(3, vec![3, 9, 9]).unwrap();
    }
    let e = open(&dir);
    assert_eq!(values(&e, 3), Some(vec![3, 9, 9]));
}

// ── PP-09 / PP-10: flushed state is reused, not re-buffered ───────────────────

#[test]
fn flushed_upserts_are_not_rebuffered_on_every_open() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        for pk in 1..=20u64 {
            e.upsert(pk, vec![pk, 1, 1]).unwrap();
        }
        e.flush().unwrap();
    }
    for _ in 0..3 {
        let e = open(&dir);
        assert_eq!(
            e.stats().rows_buffered,
            0,
            "flushed upserts were re-buffered"
        );
    }
}

#[test]
fn newest_copy_wins_when_a_pk_is_in_several_partitions() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        for v in 0..5u64 {
            e.insert(1, vec![1, v, v]).unwrap();
            e.flush().unwrap();
        }
    }
    for _ in 0..3 {
        let e = open(&dir);
        assert_eq!(values(&e, 1), Some(vec![1, 4, 4]));
    }
}

// ── PP-02 / PP-03: torn tail vs real corruption ───────────────────────────────

fn last_wal_segment(dir: &TempDir) -> std::path::PathBuf {
    let mut segs: Vec<_> = std::fs::read_dir(dir.path().join("wal"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("vwal"))
        .collect();
    segs.sort();
    segs.pop().unwrap()
}

#[test]
fn torn_wal_tail_does_not_prevent_open() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(1, vec![1, 1, 1]).unwrap();
        e.insert(2, vec![2, 2, 2]).unwrap();
    }
    // Half of a frame header: what a crash mid-append leaves.
    let seg = last_wal_segment(&dir);
    let mut f = std::fs::OpenOptions::new().append(true).open(&seg).unwrap();
    std::io::Write::write_all(&mut f, &0x5657_414Cu32.to_le_bytes()).unwrap();
    std::io::Write::write_all(&mut f, &[200, 0]).unwrap();
    drop(f);

    let e = open(&dir);
    assert_eq!(values(&e, 1), Some(vec![1, 1, 1]));
    assert_eq!(values(&e, 2), Some(vec![2, 2, 2]));
    e.insert(3, vec![3, 3, 3]).unwrap();
    drop(e);
    let e = open(&dir);
    assert_eq!(values(&e, 3), Some(vec![3, 3, 3]));
}

#[test]
fn corrupt_wal_frame_fails_open_instead_of_silently_dropping_data() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        for pk in 1..=5u64 {
            e.insert(pk, vec![pk, pk, pk]).unwrap();
        }
    }
    let seg = last_wal_segment(&dir);
    let mut bytes = std::fs::read(&seg).unwrap();
    let mid = bytes.len() / 3;
    bytes[mid] ^= 0xFF;
    std::fs::write(&seg, &bytes).unwrap();

    assert!(
        TableEngine::open_with(
            vec!["pk".into(), "age".into(), "score".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .is_err(),
        "a corrupt frame was skipped silently"
    );
}

// ── PP-13: one unreadable partition does not take the table down ──────────────

#[test]
fn unreadable_partition_is_quarantined_and_rows_recovered_from_wal() {
    let dir = TempDir::new().unwrap();
    let path;
    {
        let e = open(&dir);
        e.insert(1, vec![1, 1, 1]).unwrap();
        e.flush().unwrap();
        path = e.partition_meta()[0].path.clone();
    }
    let mut bytes = std::fs::read(&path).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(&path, &bytes).unwrap();

    let e = open(&dir);
    assert_eq!(e.quarantined_partitions().len(), 1);
    assert!(!path.exists());
    assert_eq!(
        values(&e, 1),
        Some(vec![1, 1, 1]),
        "row not rebuilt from WAL"
    );
    assert!(!e.verify().unwrap().is_clean());
}

// ── PP-07: partition ids never overwrite a live file ──────────────────────────

#[test]
fn partition_id_allocation_skips_existing_files() {
    let dir = TempDir::new().unwrap();
    let e = open(&dir);
    e.insert(1, vec![1, 1, 1]).unwrap();
    e.flush().unwrap();
    let first = e.partition_meta()[0].path.clone();
    let before = std::fs::read(&first).unwrap();
    for pk in 2..=10u64 {
        e.insert(pk, vec![pk, pk, pk]).unwrap();
        e.flush().unwrap();
    }
    assert_eq!(std::fs::read(&first).unwrap(), before);
    assert_eq!(values(&e, 1), Some(vec![1, 1, 1]));
}
