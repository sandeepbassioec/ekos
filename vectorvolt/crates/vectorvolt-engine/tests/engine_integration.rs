//! Integration tests for `vectorvolt-engine`.
//!
//! Each test builds a `TableEngine` from a temp directory, exercises the
//! write + read path, and asserts observable behaviour.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, EngineRow, TableEngine};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Two-column schema: id (PK) + age.
fn make_engine(flush_threshold: usize) -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    // WriteOnly: these tests exercise engine logic, not durability, and one fsync
    // per insert would dominate their runtime. Durability itself is covered by
    // `durability_integration.rs`.
    let engine = TableEngine::open_with(
        vec!["id".into(), "age".into()],
        dir.path(),
        EngineOptions::new(flush_threshold).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (engine, dir)
}

// ── write path ────────────────────────────────────────────────────────────────

#[test]
fn insert_increments_buffered_count() {
    let (engine, _dir) = make_engine(100);
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    assert_eq!(engine.stats().rows_buffered, 2);
    assert_eq!(engine.stats().rows_flushed, 0);
}

#[test]
fn explicit_flush_produces_partition() {
    let (engine, _dir) = make_engine(100);
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();

    let res = engine.flush().unwrap().expect("buffer should not be empty");
    assert_eq!(res.row_count, 2);

    let stats = engine.stats();
    assert_eq!(stats.rows_buffered, 0);
    assert_eq!(stats.rows_flushed, 2);
    assert_eq!(stats.partition_count, 1);
}

#[test]
fn flush_empty_returns_none() {
    let (engine, _dir) = make_engine(100);
    assert!(engine.flush().unwrap().is_none());
}

#[test]
fn auto_flush_registers_partition() {
    let (engine, _dir) = make_engine(3); // flush every 3 rows
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.insert(3, vec![3, 30]).unwrap(); // triggers auto-flush

    let stats = engine.stats();
    assert_eq!(stats.rows_buffered, 0);
    assert_eq!(stats.rows_flushed, 3);
    assert_eq!(stats.partition_count, 1);
}

#[test]
fn delete_removes_row() {
    let (engine, _dir) = make_engine(100);
    engine.insert(7, vec![7, 70]).unwrap();
    // Flush first so the row lands in a partition with a real RowDescriptor.
    engine.flush().unwrap();
    // Delete removes the pk from CuckooMap; the .vvsst file still has the row
    // bytes but the engine will never find them without the index entry.
    engine.delete(7).unwrap();

    assert!(engine.get(7).unwrap().is_none());
}

// ── read path ─────────────────────────────────────────────────────────────────

#[test]
fn get_returns_correct_values_after_explicit_flush() {
    let (engine, _dir) = make_engine(100);
    engine.insert(10, vec![10, 99]).unwrap();
    engine.insert(11, vec![11, 88]).unwrap();
    engine.flush().unwrap();

    let row = engine.get(10).unwrap().unwrap();
    assert_eq!(
        row,
        EngineRow {
            pk: 10,
            values: vec![10, 99]
        }
    );

    let row2 = engine.get(11).unwrap().unwrap();
    assert_eq!(
        row2,
        EngineRow {
            pk: 11,
            values: vec![11, 88]
        }
    );
}

#[test]
fn get_auto_flushes_pending_rows() {
    let (engine, _dir) = make_engine(1000); // high threshold — no auto-flush
    engine.insert(42, vec![42, 55]).unwrap();
    // Buffer is not flushed yet; get() must flush-on-demand.
    let row = engine.get(42).unwrap().unwrap();
    assert_eq!(row.pk, 42);
    assert_eq!(row.values[1], 55);
}

#[test]
fn get_nonexistent_returns_none() {
    let (engine, _dir) = make_engine(100);
    engine.insert(1, vec![1, 10]).unwrap();
    engine.flush().unwrap();
    assert!(engine.get(999).unwrap().is_none());
}

#[test]
fn scan_returns_matching_rows() {
    let (engine, _dir) = make_engine(100);
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.insert(3, vec![3, 30]).unwrap();
    engine.insert(4, vec![4, 40]).unwrap();
    engine.insert(5, vec![5, 50]).unwrap();

    // Scan age column (col_idx=1) for values in [20, 40].
    let results = engine.scan(1, 20, 40).unwrap();
    assert_eq!(results.len(), 3);

    let mut pks: Vec<u64> = results.iter().map(|r| r.pk).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![2, 3, 4]);
}

#[test]
fn scan_across_two_partitions() {
    let (engine, _dir) = make_engine(3);
    // First partition (rows 1-3 auto-flush on insert 3).
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 200]).unwrap();
    engine.insert(3, vec![3, 30]).unwrap();

    // Second partition.
    engine.insert(4, vec![4, 40]).unwrap();
    engine.insert(5, vec![5, 500]).unwrap();
    engine.insert(6, vec![6, 60]).unwrap();

    // Scan age in [10, 60] — should find rows 1, 3, 4, 6 (not 2 or 5).
    let results = engine.scan(1, 10, 60).unwrap();
    let mut pks: Vec<u64> = results.iter().map(|r| r.pk).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 3, 4, 6]);
}

#[test]
fn scan_col_out_of_range_errors() {
    let (engine, _dir) = make_engine(100);
    engine.insert(1, vec![1, 10]).unwrap();
    engine.flush().unwrap();
    assert!(engine.scan(99, 0, u64::MAX).is_err());
}

#[test]
fn stats_are_accurate() {
    let (engine, _dir) = make_engine(100);
    assert_eq!(engine.stats().rows_buffered, 0);
    assert_eq!(engine.stats().rows_flushed, 0);
    assert_eq!(engine.stats().partition_count, 0);

    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    assert_eq!(engine.stats().rows_buffered, 2);

    engine.flush().unwrap();
    assert_eq!(engine.stats().rows_buffered, 0);
    assert_eq!(engine.stats().rows_flushed, 2);
    assert_eq!(engine.stats().partition_count, 1);

    engine.insert(3, vec![3, 30]).unwrap();
    engine.flush().unwrap();
    assert_eq!(engine.stats().rows_flushed, 3);
    assert_eq!(engine.stats().partition_count, 2);
}

#[test]
fn schema_exposed_on_engine() {
    let (engine, _dir) = make_engine(100);
    assert_eq!(engine.schema.names(), vec!["id", "age"]);
}
