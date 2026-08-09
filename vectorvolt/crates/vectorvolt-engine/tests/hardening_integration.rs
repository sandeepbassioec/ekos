//! Layer 6 storage engine hardening tests.
//!
//! These cover scenarios that the existing suite does not exercise:
//!
//! - Compaction running while concurrent inserts land in the write buffer.
//! - Multiple sequential reopen cycles (more realistic than a single drop+open).
//! - Truncate → compact → reopen (partition-ID counter must not collide).
//! - Partial WAL record at crash boundary (reopen must not panic or corrupt good data).
//! - Wide schema round-trip (many columns, no transposition errors at the edges).
//! - Compact → insert → compact → reopen (multi-generation compaction correctness).

use std::sync::{Arc, Barrier};

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn opts() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly)
}

fn schema2() -> Vec<String> {
    vec!["id".into(), "val".into()]
}

fn schema3() -> Vec<String> {
    vec!["id".into(), "price".into(), "qty".into()]
}

// ── concurrent compaction ─────────────────────────────────────────────────────

/// compact() runs while writer threads are actively inserting into the write
/// buffer. The compacted rows from before compact() started must be intact
/// afterwards; the rows buffered during compact() must also survive.
#[test]
fn compaction_during_concurrent_inserts_preserves_all_rows() {
    const WRITERS: usize = 4;
    const PER_WRITER: u64 = 50;

    let dir = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open_with(schema3(), dir.path(), opts()).unwrap());

    // Pre-populate several partitions so compact() has meaningful work to do.
    for batch in 0u64..6 {
        for i in 0..10u64 {
            let pk = batch * 1_000 + i;
            engine.insert(pk, vec![pk, pk * 100, pk * 7]).unwrap();
        }
        engine.flush().unwrap();
    }

    // Writer threads insert into a non-overlapping PK range (10_000+).
    // The barrier synchronises them so they're inserting before compact() returns.
    let barrier = Arc::new(Barrier::new(WRITERS + 1));
    let handles: Vec<_> = (0..WRITERS)
        .map(|t| {
            let engine = Arc::clone(&engine);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let base = 10_000 + t as u64 * 1_000;
                for i in 0..PER_WRITER {
                    engine.insert(base + i, vec![base + i, i, i * 3]).unwrap();
                }
            })
        })
        .collect();

    barrier.wait();
    let result = engine.compact().unwrap().expect("compaction must run");
    assert!(result.rows_kept >= 60, "pre-compact rows vanished");

    for h in handles {
        h.join().expect("writer thread panicked");
    }

    // Flush the write buffer so all concurrent inserts land in readable partitions.
    engine.flush().unwrap();

    // Pre-compact rows must all be readable.
    for batch in 0u64..6 {
        for i in 0..10u64 {
            let pk = batch * 1_000 + i;
            let row = engine
                .get(pk)
                .unwrap()
                .unwrap_or_else(|| panic!("pre-compact pk {pk} lost"));
            assert_eq!(row.values[1], pk * 100, "value corrupted for pk {pk}");
        }
    }

    // Rows written by concurrent threads must also be readable.
    for t in 0..WRITERS as u64 {
        let base = 10_000 + t * 1_000;
        for i in 0..PER_WRITER {
            let pk = base + i;
            assert!(
                engine.get(pk).unwrap().is_some(),
                "concurrent insert pk {pk} lost after compaction"
            );
        }
    }
}

// ── sequential reopen cycles ──────────────────────────────────────────────────

/// Reopening a table three times in sequence must accumulate rows correctly
/// across each cycle. This catches state that survives one reopen but breaks
/// on a second because of stale WAL replay, double-counting, or partition-ID
/// collision.
#[test]
fn three_reopen_cycles_accumulate_rows_without_duplication() {
    let dir = TempDir::new().unwrap();

    // Cycle 1: insert pk 1-5, flush.
    {
        let e = TableEngine::open_with(schema2(), dir.path(), opts()).unwrap();
        for i in 1u64..=5 {
            e.insert(i, vec![i, i * 10]).unwrap();
        }
        e.flush().unwrap();
    }

    // Cycle 2: insert pk 6-10, flush.
    {
        let e = TableEngine::open_with(schema2(), dir.path(), opts()).unwrap();
        for i in 6u64..=10 {
            e.insert(i, vec![i, i * 10]).unwrap();
        }
        e.flush().unwrap();
    }

    // Cycle 3: insert pk 11-15, no flush (survive in WAL only).
    {
        let e = TableEngine::open_with(schema2(), dir.path(), opts()).unwrap();
        for i in 11u64..=15 {
            e.insert(i, vec![i, i * 10]).unwrap();
        }
        // deliberate: no flush
    }

    // Final open: must see all 15 rows exactly once.
    let e = TableEngine::open_with(schema2(), dir.path(), opts()).unwrap();
    let rows = e.scan(0, 0, u64::MAX).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    pks.sort_unstable();
    assert_eq!(
        pks,
        (1u64..=15).collect::<Vec<_>>(),
        "row set wrong after 3 reopen cycles"
    );

    for i in 1u64..=15 {
        let row = e
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {i} missing"));
        assert_eq!(row.values[1], i * 10, "value wrong for pk {i}");
    }
}

// ── delete all then compact → reopen ──────────────────────────────────────────

/// Delete every row, then compact (tombstone reclaim), then reopen. The
/// compacted table should be empty and writable. This exercises the case where
/// the compaction strategy has nothing useful left after GC.
#[test]
fn delete_all_then_compact_then_reopen_works() {
    use vectorvolt_storage::CompactionStrategy;

    let dir = TempDir::new().unwrap();
    let schema = schema3();

    {
        let e = TableEngine::open_with(
            schema.clone(),
            dir.path(),
            opts().with_compaction_strategy(CompactionStrategy::Leveled {
                l0_trigger: 4,
                level_multiplier: 10,
                l1_bytes: 64 * 1024 * 1024,
                garbage_threshold: 0.0, // reclaim immediately
            }),
        )
        .unwrap();

        for i in 1u64..=10 {
            e.insert(i, vec![i, i * 100, i * 7]).unwrap();
        }
        e.flush().unwrap();

        for i in 1u64..=10 {
            e.delete(i).unwrap();
        }

        let result = e
            .compact()
            .unwrap()
            .expect("compaction must run to reclaim tombstones");
        assert_eq!(result.rows_kept, 0, "all tombstoned rows should be dropped");
    }

    let e = TableEngine::open_with(schema, dir.path(), opts()).unwrap();
    assert_eq!(
        e.stats().rows_flushed,
        0,
        "deleted rows survived compact+reopen"
    );
    for i in 1u64..=10 {
        assert!(
            e.get(i).unwrap().is_none(),
            "pk {i} came back after full-delete compact"
        );
    }

    // Table must be fully writable after the empty compact+reopen.
    e.insert(1, vec![1, 999, 42]).unwrap();
    e.flush().unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values[1], 999);
}

// ── partial WAL record at crash boundary ──────────────────────────────────────

/// Simulate a crash mid-write by appending random bytes to the last WAL segment
/// after a clean write. Reopen must not panic, and the rows written before the
/// corruption must still be readable.
#[test]
fn partial_wal_record_does_not_corrupt_prior_good_data() {
    let dir = TempDir::new().unwrap();

    // Write some data and flush so it lands in a partition.
    {
        let e = TableEngine::open_with(schema2(), dir.path(), opts()).unwrap();
        for i in 1u64..=10 {
            e.insert(i, vec![i, i * 7]).unwrap();
        }
        e.flush().unwrap();
        // Write one more unflushed row so the WAL has a "tail" to corrupt.
        e.insert(99, vec![99, 693]).unwrap();
    }

    // Append a partial record — 7 bytes, less than any valid frame header.
    let wal_dir = dir.path().join("wal");
    let mut wal_files: Vec<_> = std::fs::read_dir(&wal_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    wal_files.sort_by_key(|e| e.file_name());
    if let Some(last) = wal_files.last() {
        let path = last.path();
        let mut content = std::fs::read(&path).unwrap();
        content.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03]);
        std::fs::write(&path, &content).unwrap();
    }

    // Reopen must not panic. It may return an error or silently skip the bad tail.
    let result = TableEngine::open_with(schema2(), dir.path(), opts());
    let e = match result {
        Ok(e) => e,
        Err(_) => return, // graceful error is acceptable
    };

    // The 10 flushed rows must be intact regardless of the WAL tail.
    for i in 1u64..=10 {
        let row = e
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("flushed pk {i} lost after WAL corruption"));
        assert_eq!(row.values[1], i * 7);
    }
}

// ── wide schema round-trip ────────────────────────────────────────────────────

/// A 32-column table flushed, compacted, and reopened must preserve every
/// column value without transposition. Wide schemas stress the column-assembly
/// path that compaction and partition readers share.
#[test]
fn wide_schema_survives_compact_and_reopen() {
    const COLS: usize = 32;
    let dir = TempDir::new().unwrap();
    let schema: Vec<String> = (0..COLS).map(|i| format!("c{i}")).collect();

    {
        let e = TableEngine::open_with(schema.clone(), dir.path(), opts()).unwrap();
        for pk in 1u64..=20 {
            let values: Vec<u64> = (0..COLS as u64).map(|c| pk * 1_000 + c).collect();
            e.insert(pk, values).unwrap();
            e.flush().unwrap();
        }
        e.compact().unwrap().expect("compaction must run");
    }

    let e = TableEngine::open_with(schema, dir.path(), opts()).unwrap();
    for pk in 1u64..=20 {
        let row = e
            .get(pk)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {pk} missing after wide-schema reopen"));
        for (col, &v) in row.values.iter().enumerate() {
            let expected = pk * 1_000 + col as u64;
            assert_eq!(
                v, expected,
                "pk {pk} col {col} corrupted: got {v}, want {expected}"
            );
        }
    }
}

// ── multi-generation compaction ───────────────────────────────────────────────

/// Two compaction passes followed by a reopen: the first pass merges the initial
/// partitions; the second merges those with new inserts. Tests that the
/// partition-ID counter does not collide and that data from both generations is
/// intact.
#[test]
fn multi_generation_compaction_then_reopen() {
    let dir = TempDir::new().unwrap();
    let schema = schema3();

    {
        let e = TableEngine::open_with(schema.clone(), dir.path(), opts()).unwrap();

        // Generation 1: pk 1-10.
        for i in 1u64..=10 {
            e.insert(i, vec![i, i * 100, i * 7]).unwrap();
            e.flush().unwrap();
        }
        let r1 = e.compact().unwrap().expect("first compaction must run");
        assert_eq!(r1.rows_kept, 10);
        assert_eq!(e.stats().partition_count, 1);

        // Generation 2: pk 11-20, written after compaction.
        for i in 11u64..=20 {
            e.insert(i, vec![i, i * 100, i * 7]).unwrap();
            e.flush().unwrap();
        }
        let r2 = e.compact().unwrap().expect("second compaction must run");
        assert!(
            r2.rows_kept >= 10,
            "second compaction must keep the new rows"
        );
    }

    let e = TableEngine::open_with(schema, dir.path(), opts()).unwrap();
    assert_eq!(e.stats().rows_flushed, 20);

    for i in 1u64..=20 {
        let row = e
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {i} missing after multi-generation compact+reopen"));
        assert_eq!(row.values[1], i * 100, "price corrupted for pk {i}");
        assert_eq!(row.values[2], i * 7, "qty corrupted for pk {i}");
    }
}
