//! VV-QRY-002.6 — range scans on the PK column use a binary search within a
//! partition once it is PK-sorted, instead of a full linear scan, while still
//! returning exactly the same rows a linear scan would.
//!
//! Two partition shapes matter:
//! - Freshly flushed (pre-compaction): rows land in whatever order
//!   `TxnManager::flush` drained them, NOT necessarily PK order.
//! - Post-compaction: `compaction::execute_job` always emits PK-sorted output.
//!
//! `TableEngine::scan(0, lo, hi)` must return identical results either way —
//! these tests insert rows out of PK order specifically to make sure the
//! binary-search path (taken only when the partition turns out sorted) never
//! gets exercised by accident on data that isn't, and is exercised for real
//! once compaction sorts it.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn fast() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly)
}

fn make_engine() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine =
        TableEngine::open_with(vec!["pk".into(), "val".into()], dir.path(), fast()).unwrap();
    (engine, dir)
}

fn pks_of(rows: &[vectorvolt_engine::EngineRow]) -> Vec<u64> {
    let mut v: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    v.sort_unstable();
    v
}

// ── pre-compaction: rows inserted out of PK order, single flush ─────────────

#[test]
fn unsorted_flush_partition_range_scan_partial_overlap_start() {
    let (eng, _dir) = make_engine();
    // Insert in a shuffled, non-monotonic PK order.
    for pk in [50u64, 10, 30, 90, 20, 70, 60, 40, 80, 0] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan(0, 0, 25).unwrap();
    assert_eq!(pks_of(&rows), vec![0, 10, 20]);
}

#[test]
fn unsorted_flush_partition_range_scan_partial_overlap_end() {
    let (eng, _dir) = make_engine();
    for pk in [50u64, 10, 30, 90, 20, 70, 60, 40, 80, 0] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan(0, 65, 200).unwrap();
    assert_eq!(pks_of(&rows), vec![70, 80, 90]);
}

#[test]
fn unsorted_flush_partition_range_scan_empty_and_whole() {
    let (eng, _dir) = make_engine();
    for pk in [50u64, 10, 30, 90, 20, 70, 60, 40, 80, 0] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();

    assert!(eng.scan(0, 1000, 2000).unwrap().is_empty());

    let all = eng.scan(0, 0, 90).unwrap();
    assert_eq!(all.len(), 10);
}

#[test]
fn unsorted_flush_partition_single_match() {
    let (eng, _dir) = make_engine();
    for pk in [50u64, 10, 30, 90, 20, 70, 60, 40, 80, 0] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan(0, 30, 30).unwrap();
    assert_eq!(pks_of(&rows), vec![30]);
}

// ── post-compaction: output is guaranteed PK-sorted ──────────────────────────

#[test]
fn compacted_partition_range_scan_boundary_cases() {
    let (eng, _dir) = make_engine();
    // Two flushes out of PK order, then compact so the merged output is sorted.
    for pk in [8u64, 3, 6, 1, 9, 4] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();
    for pk in [7u64, 2, 5, 0] {
        eng.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    eng.flush().unwrap();
    eng.compact().unwrap();

    // Range start exactly at first row.
    assert_eq!(pks_of(&eng.scan(0, 0, 0).unwrap()), vec![0]);
    // Range end exactly at last row.
    assert_eq!(pks_of(&eng.scan(0, 9, 9).unwrap()), vec![9]);
    // Partial overlap at start.
    assert_eq!(pks_of(&eng.scan(0, 0, 3).unwrap()), vec![0, 1, 2, 3]);
    // Partial overlap at end.
    assert_eq!(pks_of(&eng.scan(0, 7, 100).unwrap()), vec![7, 8, 9]);
    // Empty match.
    assert!(eng.scan(0, 100, 200).unwrap().is_empty());
    // Whole-partition match.
    assert_eq!(eng.scan(0, 0, 9).unwrap().len(), 10);
    // Inverted range.
    assert!(eng.scan(0, 5, 2).unwrap().is_empty());
}

#[test]
fn compacted_partition_large_tight_range_matches_full_scan() {
    let (eng, _dir) = make_engine();
    // Insert 2,000 rows in reverse PK order across a few flushes, then compact.
    for chunk in (0u64..2000).collect::<Vec<_>>().rchunks(400) {
        for &pk in chunk.iter().rev() {
            eng.insert(pk, vec![pk, pk + 1]).unwrap();
        }
        eng.flush().unwrap();
    }
    eng.compact().unwrap();

    let rows = eng.scan(0, 995, 1005).unwrap();
    assert_eq!(pks_of(&rows), (995u64..=1005).collect::<Vec<_>>());

    // Cross-check against an unbounded scan filtered manually — proves the
    // range-scan path returns the exact same rows a full scan + predicate
    // check would.
    let all = eng.scan(0, 0, u64::MAX).unwrap();
    let mut expected: Vec<u64> = all
        .iter()
        .filter(|r| r.pk >= 995 && r.pk <= 1005)
        .map(|r| r.pk)
        .collect();
    expected.sort_unstable();
    assert_eq!(pks_of(&rows), expected);
}

// ── scan_where: first predicate on the PK column also benefits ──────────────

#[test]
fn scan_where_pk_first_predicate_matches_linear_semantics() {
    let (eng, _dir) = make_engine();
    for pk in [8u64, 3, 6, 1, 9, 4, 7, 2, 5, 0] {
        eng.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    eng.flush().unwrap();
    eng.compact().unwrap();

    // pk in [3, 7] AND val in [40, 60] -> pk in {4,5,6}
    let rows = eng.scan_where(&[(0, 3, 7), (1, 40, 60)]).unwrap();
    assert_eq!(pks_of(&rows), vec![4, 5, 6]);
}
