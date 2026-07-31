//! The partition lifecycle through `TableEngine` (VV-STO-015/016/020).
//!
//! Compaction, retention, and verification were reachable only from `TxnManager`.
//! These cover them at the level a user actually holds, plus the two behaviours
//! that only exist once the engine owns a manifest: plans that can be inspected
//! before they run, and recovery state that is reported rather than swallowed.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_storage::{RetentionPolicy, RetentionRule, RetentionVeto};

fn schema() -> Vec<String> {
    vec!["id".into(), "price".into(), "qty".into()]
}

fn engine(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        schema(),
        dir.path(),
        EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
    )
    .unwrap()
}

fn fill(e: &TableEngine, base: u64, n: u64) {
    for i in base..base + n {
        e.insert(i, vec![i, i * 100, i * 7]).unwrap();
    }
    e.flush().unwrap();
}

/// The manifest tracks what the engine flushes, with no opt-in.
#[test]
fn engine_records_partitions_automatically() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    assert!(e.partition_meta().is_empty());
    assert!(e.manifest_recovery().is_clean());

    for b in 0..3u64 {
        fill(&e, b * 100, 10);
    }
    let parts = e.partition_meta();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts.iter().map(|p| p.row_count).sum::<u32>(), 30);
    assert!(parts.iter().all(|p| p.level == 0));
    assert!(parts.iter().all(|p| p.size_bytes > 0));
}

/// A plan can be inspected before it is executed.
///
/// Compaction destroys data. Being able to see what it would do without doing it
/// is the difference between an operator being able to reason about a production
/// table and having to find out afterwards.
#[test]
fn compaction_plan_is_inspectable_and_matches_what_runs() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    for b in 0..6u64 {
        fill(&e, b * 100, 10);
    }

    let plan = e.compaction_plan();
    assert_eq!(plan.len(), 1, "6 L0 partitions should plan one merge");
    assert_eq!(plan[0].inputs.len(), 6);
    let planned_rows = plan[0].input_rows();

    // Planning is pure: nothing changed.
    assert_eq!(e.partition_meta().len(), 6, "the plan mutated state");

    let result = e.compact().unwrap().expect("compaction should run");
    assert_eq!(result.rows_kept as u64, planned_rows);
    assert_eq!(e.partition_meta().len(), 1);
    assert!(e.compaction_plan().is_empty(), "nothing left to plan");
}

/// Compaction leaves the table fully readable and reports no leaks.
#[test]
fn compaction_through_the_engine_preserves_every_row() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    for b in 0..5u64 {
        fill(&e, b * 100, 12);
    }
    e.delete(203).unwrap();

    let result = e.compact().unwrap().unwrap();
    assert_eq!(result.rows_kept, 59, "60 inserted, 1 deleted");
    assert_eq!(
        result.leaked_files, 0,
        "input partitions were not deleted — a mapping was still live"
    );

    for b in 0..5u64 {
        for i in 0..12u64 {
            let pk = b * 100 + i;
            let got = e.get(pk).unwrap();
            if pk == 203 {
                assert!(got.is_none(), "deleted pk came back");
            } else {
                let row = got.unwrap_or_else(|| panic!("pk {pk} lost"));
                assert_eq!(row.values[1], pk * 100, "pk {pk} value corrupted");
                assert_eq!(row.values[2], pk * 7);
            }
        }
    }
    assert!(e.verify().unwrap().is_clean());
}

/// Retention deletes partitions and the rows in them stop resolving.
#[test]
fn retention_deletes_partitions_and_prunes_the_index() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    for b in 0..5u64 {
        fill(&e, b * 100, 10);
    }
    assert_eq!(e.partition_meta().len(), 5);

    let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxPartitions(2));
    // Dry run first — must not change anything.
    let dry = e.retention_plan(&policy, u64::MAX / 2);
    assert_eq!(dry.delete.len(), 3);
    assert_eq!(e.partition_meta().len(), 5, "retention_plan mutated state");

    let applied = e.apply_retention(&policy, u64::MAX / 2).unwrap();
    assert_eq!(applied.delete.len(), 3);
    assert!(applied.undeleted.is_empty(), "files could not be removed");
    assert_eq!(e.partition_meta().len(), 2);

    // Rows in dropped partitions must stop resolving — an index entry pointing at
    // a deleted file is worse than a missing row.
    for b in 0..3u64 {
        for i in 0..10u64 {
            assert!(
                e.get(b * 100 + i).unwrap().is_none(),
                "pk {} survived its partition",
                b * 100 + i
            );
        }
    }
    // And the survivors are intact.
    for b in 3..5u64 {
        for i in 0..10u64 {
            let pk = b * 100 + i;
            assert_eq!(e.get(pk).unwrap().unwrap().values[1], pk * 100);
        }
    }
    assert!(e.verify().unwrap().is_clean());
}

/// The last partition is never deleted, and the veto is reported.
#[test]
fn retention_refuses_to_empty_the_table() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    for b in 0..3u64 {
        fill(&e, b * 100, 5);
    }
    let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxTotalBytes(0));
    let applied = e.apply_retention(&policy, u64::MAX / 2).unwrap();

    assert_eq!(applied.delete.len(), 2);
    assert!(
        applied
            .retained
            .iter()
            .any(|(_, v)| *v == RetentionVeto::LastPartition),
        "the rail must be reported, not silently applied"
    );
    assert_eq!(e.partition_meta().len(), 1);
    assert!(e.verify().unwrap().is_clean());
}

/// A policy that selects nothing does nothing.
#[test]
fn retention_with_nothing_to_do_is_a_noop() {
    let dir = TempDir::new().unwrap();
    let e = engine(&dir);
    for b in 0..3u64 {
        fill(&e, b * 100, 5);
    }
    let before = e.partition_meta();
    let applied = e
        .apply_retention(
            &RetentionPolicy::new().with_rule(RetentionRule::MaxPartitions(10)),
            u64::MAX / 2,
        )
        .unwrap();
    assert!(applied.delete.is_empty());
    assert_eq!(e.partition_meta(), before);
    // Rows still resolve — a no-op must not rebuild the index into nothing.
    assert!(e.get(0).unwrap().is_some());
}

/// A table written before the manifest existed is adopted on open.
///
/// Partitions on disk that the manifest does not name are only adopted when the
/// manifest is EMPTY. That distinction matters: with a populated manifest an
/// unknown file is an orphan from an interrupted compaction, and adopting it
/// would resurrect rows compaction deliberately removed.
#[test]
fn pre_manifest_partitions_are_adopted_on_open() {
    let dir = TempDir::new().unwrap();
    {
        let e = engine(&dir);
        for b in 0..3u64 {
            fill(&e, b * 100, 8);
        }
    }
    // Simulate a table from before the manifest: delete it and reopen.
    let manifest = dir.path().join("data").join("MANIFEST");
    assert!(manifest.exists());
    std::fs::remove_file(&manifest).unwrap();

    let e = engine(&dir);
    assert_eq!(
        e.partition_meta().len(),
        3,
        "existing partitions were not adopted into a fresh manifest"
    );
    for b in 0..3u64 {
        for i in 0..8u64 {
            assert!(e.get(b * 100 + i).unwrap().is_some());
        }
    }
    assert!(e.verify().unwrap().is_clean());
}

/// An orphan left beside a populated manifest is NOT adopted.
///
/// The counterpart to the test above, and the one that matters more: an
/// interrupted compaction leaves its inputs on disk with the manifest already
/// naming only the output. Adopting them would double every row they hold.
#[test]
fn orphans_beside_a_populated_manifest_are_ignored() {
    let dir = TempDir::new().unwrap();
    let orphan;
    {
        let e = engine(&dir);
        for b in 0..6u64 {
            fill(&e, b * 100, 5);
        }
        e.compact().unwrap().unwrap();
        assert_eq!(e.partition_meta().len(), 1);

        // Fabricate an orphan: copy the live partition under an unused id, exactly
        // as an interrupted compaction would have left one behind.
        let live = e.partition_meta()[0].clone();
        orphan = dir.path().join("data").join("00000_09999.vvsst");
        std::fs::copy(&live.path, &orphan).unwrap();
    }

    let e = engine(&dir);
    assert_eq!(
        e.partition_meta().len(),
        1,
        "an orphan was adopted as a live partition"
    );
    assert_eq!(e.stats().rows_flushed, 30, "rows were double-counted");
    assert!(
        orphan.exists(),
        "the orphan should be left alone, not deleted"
    );
}

/// Verification reports a real problem rather than passing everything.
#[test]
fn verify_detects_a_damaged_partition() {
    let dir = TempDir::new().unwrap();
    let path;
    {
        let e = engine(&dir);
        for b in 0..3u64 {
            fill(&e, b * 100, 5);
        }
        assert!(e.verify().unwrap().is_clean());
        path = e.partition_meta()[0].path.clone();
    }

    // Flip a byte in the payload. The whole-file CRC must catch it.
    let mut bytes = std::fs::read(&path).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(&path, &bytes).unwrap();

    let e = TableEngine::open_with(
        schema(),
        dir.path(),
        EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
    );
    // Opening may fail outright (the reader verifies CRCs); if it does not, verify
    // must report the damage. Either is acceptable — silence is not.
    match e {
        Err(_) => {}
        Ok(e) => {
            let report = e.verify().unwrap();
            assert!(!report.is_clean(), "a corrupted partition verified clean");
        }
    }
}
