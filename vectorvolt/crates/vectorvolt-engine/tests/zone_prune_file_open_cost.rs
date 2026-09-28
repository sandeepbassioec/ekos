//! VV-ENG-014: zone pruning must save file-open cost, not just scan work.
//!
//! `PartitionReader::open` performs a full-file CRC32 and (on Linux) a full
//! `mmap` page prefault — cost proportional to the whole partition, paid
//! whether or not a single row is ever read from it. Before this fix, every
//! partition discovered by `sync_partitions` (i.e. every flush) paid that cost
//! immediately, regardless of whether any future query's zone-map filter would
//! exclude it. These tests make that cost observable via
//! `TableEngine::partition_open_count` (a counter of real `PartitionReader::open`
//! calls) and prove correctness is unchanged.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn make_engine(flush_threshold: usize) -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = TableEngine::open_with(
        vec!["id".into(), "age".into()],
        dir.path(),
        EngineOptions::new(flush_threshold).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (engine, dir)
}

/// A scan whose predicate a zone map excludes entirely must never open the
/// excluded partition's file.
#[test]
fn zone_pruned_partition_is_never_opened() {
    let (engine, _dir) = make_engine(2);

    // Partition A: ages 10..=20.
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.flush().unwrap();

    // Partition B: ages 100..=110.
    engine.insert(3, vec![3, 100]).unwrap();
    engine.insert(4, vec![4, 110]).unwrap();
    engine.flush().unwrap();

    assert_eq!(engine.stats().partition_count, 2);
    // Neither partition has been opened yet — only registered via the cheap
    // zone-map peek that `sync_partitions` performs after each flush.
    assert_eq!(engine.partition_open_count(), 0);

    // This range only overlaps partition A's zone map [10, 20].
    let rows = engine.scan(1, 10, 20).unwrap();
    assert_eq!(rows.len(), 2);

    // Exactly one partition should have been opened (the survivor). If the old
    // ordering regressed, both would be open here.
    assert_eq!(
        engine.partition_open_count(),
        1,
        "zone-excluded partition B must not be opened by a scan whose range only overlaps A"
    );

    // Scanning the excluded range now must not open anything new either.
    let rows = engine.scan(1, 100, 110).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        engine.partition_open_count(),
        2,
        "partition B is opened once it is actually the survivor of a query"
    );
}

/// A predicate that excludes every partition must open none of them.
#[test]
fn scan_excluding_all_partitions_opens_nothing() {
    let (engine, _dir) = make_engine(2);

    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.flush().unwrap();

    engine.insert(3, vec![3, 100]).unwrap();
    engine.insert(4, vec![4, 110]).unwrap();
    engine.flush().unwrap();

    assert_eq!(engine.partition_open_count(), 0);

    let rows = engine.scan(1, 500, 600).unwrap();
    assert!(rows.is_empty());
    assert_eq!(
        engine.partition_open_count(),
        0,
        "a range overlapping no partition's zone map must open none of them"
    );
}

/// Existing zone-pruning correctness: matching rows are still returned
/// correctly, including at partial-overlap and exact-boundary values.
#[test]
fn zone_pruning_correctness_boundaries_and_partial_overlap() {
    let (engine, _dir) = make_engine(2);

    // Partition A: ages [10, 20].
    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.flush().unwrap();

    // Partition B: ages [20, 30] — shares the boundary value 20 with A.
    engine.insert(3, vec![3, 20]).unwrap();
    engine.insert(4, vec![4, 30]).unwrap();
    engine.flush().unwrap();

    // Exact lower boundary of A only.
    let mut pks: Vec<u64> = engine
        .scan(1, 10, 10)
        .unwrap()
        .into_iter()
        .map(|r| r.pk)
        .collect();
    pks.sort();
    assert_eq!(pks, vec![1]);

    // Exact shared boundary value: must return rows from BOTH partitions.
    let mut pks: Vec<u64> = engine
        .scan(1, 20, 20)
        .unwrap()
        .into_iter()
        .map(|r| r.pk)
        .collect();
    pks.sort();
    assert_eq!(pks, vec![2, 3]);

    // Partial overlap: [15, 25] touches the top of A and the bottom of B.
    let mut pks: Vec<u64> = engine
        .scan(1, 15, 25)
        .unwrap()
        .into_iter()
        .map(|r| r.pk)
        .collect();
    pks.sort();
    assert_eq!(pks, vec![2, 3]);

    // Full range covers everything.
    let mut pks: Vec<u64> = engine
        .scan(1, 0, 1000)
        .unwrap()
        .into_iter()
        .map(|r| r.pk)
        .collect();
    pks.sort();
    assert_eq!(pks, vec![1, 2, 3, 4]);
}

/// `get()` addresses a specific partition directly and must still work when
/// that partition has only been zone-map-peeked, not yet opened.
#[test]
fn get_promotes_a_pending_partition() {
    let (engine, _dir) = make_engine(2);

    engine.insert(1, vec![1, 10]).unwrap();
    engine.insert(2, vec![2, 20]).unwrap();
    engine.flush().unwrap();

    assert_eq!(engine.partition_open_count(), 0);

    let row = engine.get(1).unwrap().expect("row must be found");
    assert_eq!(row.values, vec![1, 10]);
    assert_eq!(
        engine.partition_open_count(),
        1,
        "a direct point lookup has no predicate to prune with, so it must open its partition"
    );
}
