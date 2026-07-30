//! Integration tests for [`TableEngine::compact`].
//!
//! Compaction is the most destructive operation in the crate: it **deletes
//! partition files** and **rebuilds the primary-key index**. A bug here does not
//! degrade performance, it loses or corrupts data. These tests exist because the
//! feature originally shipped verified only by hand through the CLI, which proves
//! it worked once and guarantees nothing thereafter.
//!
//! The two that matter most:
//!
//! - `compaction_preserves_every_column_value` — `compact()` disassembles rows
//!   into columns and reassembles them. An off-by-one in that transposition would
//!   silently pair the wrong value with the wrong PK, and a test that only checked
//!   "the PK is still findable" would pass while the data was scrambled.
//! - `compacted_table_survives_reopen` — compaction assigns new partition IDs and
//!   deletes the old files. If the reopen path cannot rebuild the index from what
//!   is left on disk, the data is unreachable after a restart.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

/// Non-durable options, for tests that are not about durability.
fn fast() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly)
}

/// Three-column schema: id (PK), price, qty.
fn make_engine(flush_threshold: usize) -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    // WriteOnly — see the note in engine_integration.rs.
    let engine = TableEngine::open_with(
        vec!["id".into(), "price".into(), "qty".into()],
        dir.path(),
        EngineOptions::new(flush_threshold).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (engine, dir)
}

/// Insert `n` rows, flushing after each so every row lands in its own partition —
/// which is exactly what the CLI does, and the case compaction exists to fix.
fn insert_one_per_partition(engine: &TableEngine, n: u64) {
    for i in 1..=n {
        engine.insert(i, vec![i, i * 100, i * 7]).unwrap();
        engine.flush().unwrap();
    }
}

// ── the core promise ──────────────────────────────────────────────────────────

#[test]
fn compaction_merges_many_partitions_into_one() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 10);
    assert_eq!(
        engine.stats().partition_count,
        10,
        "setup: expected 10 partitions"
    );

    let result = engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    assert_eq!(result.rows_kept, 10);
    assert_eq!(
        engine.stats().partition_count,
        1,
        "should collapse to one partition"
    );
    assert_eq!(engine.stats().rows_flushed, 10);
}

#[test]
fn compaction_preserves_every_column_value() {
    // The transposition test. compact() reads columns and rebuilds rows; an
    // off-by-one would misalign values against PKs while leaving every PK findable.
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 25);

    engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    for i in 1..=25u64 {
        let row = engine
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {i} vanished after compaction"));
        assert_eq!(row.pk, i);
        assert_eq!(row.values[0], i, "pk column wrong for pk {i}");
        assert_eq!(row.values[1], i * 100, "price misaligned for pk {i}");
        assert_eq!(row.values[2], i * 7, "qty misaligned for pk {i}");
    }
}

#[test]
fn compaction_output_is_pk_sorted() {
    // Sorted output is what makes zone-map pruning effective on the merged
    // partition; unsorted output would still be *correct* but would silently
    // undo the pruning benefit, so assert the ordering explicitly.
    let (engine, _dir) = make_engine(1_000_000);
    // Insert deliberately out of order.
    for i in [7u64, 2, 9, 1, 5, 3, 8, 4, 6] {
        engine.insert(i, vec![i, i * 100, i * 7]).unwrap();
        engine.flush().unwrap();
    }

    engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    let rows = engine.scan(0, 0, u64::MAX).unwrap();
    let pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    let mut sorted = pks.clone();
    sorted.sort_unstable();
    assert_eq!(pks, sorted, "compacted partition is not PK-sorted");
}

// ── tombstone reclaim ─────────────────────────────────────────────────────────

#[test]
fn compaction_reclaims_deleted_rows() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 10);

    engine.delete(3).unwrap();
    engine.delete(7).unwrap();

    let result = engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    assert_eq!(
        result.rows_kept, 8,
        "deleted rows should not be carried over"
    );
    assert_eq!(engine.stats().rows_flushed, 8);
    assert!(
        engine.get(3).unwrap().is_none(),
        "pk 3 came back from the dead"
    );
    assert!(
        engine.get(7).unwrap().is_none(),
        "pk 7 came back from the dead"
    );
    // And the survivors are intact.
    for i in [1u64, 2, 4, 5, 6, 8, 9, 10] {
        assert!(engine.get(i).unwrap().is_some(), "pk {i} was lost");
    }
}

#[test]
fn compaction_reclaims_from_a_single_partition() {
    // The "already one partition" early-return must not skip tombstone reclaim.
    let (engine, _dir) = make_engine(1_000_000);
    for i in 1..=5u64 {
        engine.insert(i, vec![i, i * 100, i * 7]).unwrap();
    }
    engine.flush().unwrap();
    assert_eq!(engine.stats().partition_count, 1);

    engine.delete(2).unwrap();

    let result = engine
        .compact()
        .unwrap()
        .expect("must still compact to reclaim the tombstone, despite 1 partition");
    assert_eq!(result.rows_kept, 4);
    assert!(engine.get(2).unwrap().is_none());
}

// ── no-op cases ───────────────────────────────────────────────────────────────

#[test]
fn compacting_an_empty_table_is_a_noop() {
    let (engine, _dir) = make_engine(1_000_000);
    assert!(engine.compact().unwrap().is_none());
}

#[test]
fn compacting_an_already_optimal_table_is_a_noop() {
    let (engine, _dir) = make_engine(1_000_000);
    for i in 1..=5u64 {
        engine.insert(i, vec![i, i * 100, i * 7]).unwrap();
    }
    engine.flush().unwrap();

    // One partition, no tombstones — nothing to gain.
    assert!(
        engine.compact().unwrap().is_none(),
        "should not rewrite a table that is already optimal"
    );
}

#[test]
fn compaction_is_idempotent() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 6);

    let first = engine
        .compact()
        .unwrap()
        .expect("first compaction should run");
    assert_eq!(first.rows_kept, 6);

    // Second run has nothing left to do.
    assert!(
        engine.compact().unwrap().is_none(),
        "compaction is not idempotent — it rewrote an already-compacted table"
    );
    assert_eq!(engine.stats().partition_count, 1);
    assert_eq!(engine.stats().rows_flushed, 6);
}

// ── durability ────────────────────────────────────────────────────────────────

#[test]
fn compacted_table_survives_reopen() {
    // Compaction assigns new partition IDs and deletes the old files. If the
    // reopen path cannot rebuild the index from what remains, the data is gone.
    let dir = TempDir::new().unwrap();
    let schema = vec!["id".to_string(), "price".to_string(), "qty".to_string()];

    {
        let engine = TableEngine::open_with(schema.clone(), dir.path(), fast()).unwrap();
        insert_one_per_partition(&engine, 12);
        engine.delete(5).unwrap();
        let r = engine
            .compact()
            .unwrap()
            .expect("compaction should have run");
        assert_eq!(r.rows_kept, 11);
    } // engine dropped — mmaps released, process boundary simulated

    let reopened = TableEngine::open_with(schema, dir.path(), fast()).unwrap();
    assert_eq!(
        reopened.stats().partition_count,
        1,
        "reopen did not find the merged partition"
    );
    assert_eq!(reopened.stats().rows_flushed, 11);

    for i in 1..=12u64 {
        let got = reopened.get(i).unwrap();
        if i == 5 {
            assert!(got.is_none(), "deleted pk 5 resurrected across reopen");
        } else {
            let row = got.unwrap_or_else(|| panic!("pk {i} lost across reopen"));
            assert_eq!(
                row.values[1],
                i * 100,
                "price corrupted across reopen for pk {i}"
            );
            assert_eq!(
                row.values[2],
                i * 7,
                "qty corrupted across reopen for pk {i}"
            );
        }
    }
}

#[test]
fn compaction_deletes_the_old_partition_files() {
    // The point of compaction is reclaiming space. If old files linger, we have
    // merely written more data.
    let (engine, dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 8);

    let data_dir = dir.path().join("data");
    let before = std::fs::read_dir(&data_dir).unwrap().count();
    assert_eq!(before, 8, "setup: expected 8 .vvsst files");

    engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    let after: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("vvsst"))
        .collect();
    assert_eq!(
        after.len(),
        1,
        "old partition files were not deleted — {} remain",
        after.len()
    );
}

// ── interaction with other operations ─────────────────────────────────────────

#[test]
fn writes_after_compaction_still_work() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 5);
    engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    // The partition-id counter must not collide with the merged partition.
    engine.insert(100, vec![100, 999, 42]).unwrap();
    engine.flush().unwrap();

    let row = engine
        .get(100)
        .unwrap()
        .expect("post-compaction insert lost");
    assert_eq!(row.values[1], 999);
    // And the pre-compaction rows are untouched.
    assert_eq!(engine.get(3).unwrap().unwrap().values[1], 300);
}

#[test]
fn compaction_flushes_pending_buffer_first() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 3);
    // Leave these in the write buffer, unflushed.
    engine.insert(50, vec![50, 5000, 350]).unwrap();
    engine.insert(51, vec![51, 5100, 357]).unwrap();
    assert_eq!(engine.stats().rows_buffered, 2);

    let result = engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    assert_eq!(
        result.rows_kept, 5,
        "buffered rows were dropped by compaction"
    );
    assert_eq!(engine.stats().rows_buffered, 0);
    assert_eq!(engine.get(50).unwrap().unwrap().values[1], 5000);
}

#[test]
fn scan_works_after_compaction() {
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 20);
    engine.delete(10).unwrap();
    engine
        .compact()
        .unwrap()
        .expect("compaction should have run");

    // price = i * 100, so [500, 1500] covers pks 5..=15, minus deleted 10.
    let rows = engine.scan(1, 500, 1500).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![5, 6, 7, 8, 9, 11, 12, 13, 14, 15]);
}

#[test]
fn compaction_handles_a_larger_partition_count() {
    // Exercises the merge path with enough partitions that an off-by-one in the
    // offset arithmetic would surface.
    let (engine, _dir) = make_engine(1_000_000);
    insert_one_per_partition(&engine, 120);

    let result = engine
        .compact()
        .unwrap()
        .expect("compaction should have run");
    assert_eq!(result.rows_kept, 120);
    assert_eq!(engine.stats().partition_count, 1);

    // Spot-check the boundaries and the middle.
    for i in [1u64, 2, 59, 60, 61, 119, 120] {
        let row = engine
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {i} lost"));
        assert_eq!(row.values[1], i * 100, "value misaligned at pk {i}");
    }
}
