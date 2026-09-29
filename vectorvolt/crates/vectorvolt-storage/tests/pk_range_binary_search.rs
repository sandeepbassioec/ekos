//! VV-QRY-002.6 — binary search within a partition for PK range scans.
//!
//! `PartitionReader::filter_offsets_pk` must return exactly the same offsets
//! as `filter_offsets(0, lo, hi)` (the old linear/SIMD path) whether or not
//! column 0 happens to be sorted, and must correctly detect sortedness via
//! `is_pk_sorted`.

use std::io::Write;
use tempfile::NamedTempFile;
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

fn reader_for(pks: Vec<u64>) -> (NamedTempFile, PartitionReader) {
    let mut p = MicroPartition::new(1, 1);
    // Column 0 stored Raw, matching how the engine always stores the PK
    // column (see TxnManager::flush / compaction::execute_job).
    p.push_column("pk", ColumnValues::Raw(pks)).unwrap();
    let mut f = NamedTempFile::new().unwrap();
    write_partition(&mut f, &p).unwrap();
    f.flush().unwrap();
    let r = PartitionReader::open(f.path()).unwrap();
    (f, r)
}

fn assert_matches_linear_scan(r: &PartitionReader, lo: u64, hi: u64) {
    let expected = r.filter_offsets(0, lo, hi).unwrap();
    let actual = r.filter_offsets_pk(lo, hi).unwrap();
    assert_eq!(
        actual, expected,
        "filter_offsets_pk disagrees with linear scan for [{lo}, {hi}]"
    );
}

// ── sortedness detection ─────────────────────────────────────────────────────

#[test]
fn is_pk_sorted_true_for_sorted_pks() {
    let (_f, r) = reader_for((0u64..1000).collect());
    assert!(r.is_pk_sorted().unwrap());
}

#[test]
fn is_pk_sorted_false_for_insertion_order_pks() {
    // Same as a fresh memtable flush would produce: no sort applied.
    let (_f, r) = reader_for(vec![5, 1, 9, 2, 8, 0, 7, 3, 6, 4]);
    assert!(!r.is_pk_sorted().unwrap());
}

#[test]
fn is_pk_sorted_true_for_empty_and_single_row() {
    let (_f, r) = reader_for(vec![]);
    assert!(r.is_pk_sorted().unwrap());
    let (_f2, r2) = reader_for(vec![42]);
    assert!(r2.is_pk_sorted().unwrap());
}

#[test]
fn is_pk_sorted_true_for_duplicate_values() {
    // Non-decreasing, not strictly increasing — duplicates are still "sorted".
    let (_f, r) = reader_for(vec![1, 1, 2, 2, 2, 3]);
    assert!(r.is_pk_sorted().unwrap());
}

#[test]
fn is_pk_sorted_cached_across_calls() {
    let (_f, r) = reader_for((0u64..500).collect());
    assert!(r.is_pk_sorted().unwrap());
    // Second call must hit the cache and agree.
    assert!(r.is_pk_sorted().unwrap());
}

// ── correctness on a SORTED partition (binary search path) ──────────────────

#[test]
fn sorted_partition_partial_overlap_start() {
    let (_f, r) = reader_for((10u64..20).collect()); // 10..=19
    assert_matches_linear_scan(&r, 5, 12); // overlaps the start
    let offsets = r.filter_offsets_pk(5, 12).unwrap();
    assert_eq!(offsets, vec![0, 1, 2]); // pks 10, 11, 12
}

#[test]
fn sorted_partition_partial_overlap_end() {
    let (_f, r) = reader_for((10u64..20).collect());
    assert_matches_linear_scan(&r, 17, 100);
    let offsets = r.filter_offsets_pk(17, 100).unwrap();
    assert_eq!(offsets, vec![7, 8, 9]); // pks 17, 18, 19
}

#[test]
fn sorted_partition_whole_range_matches() {
    let (_f, r) = reader_for((0u64..1000).collect());
    assert_matches_linear_scan(&r, 0, 999);
    assert_eq!(r.filter_offsets_pk(0, 999).unwrap().len(), 1000);
}

#[test]
fn sorted_partition_empty_result() {
    let (_f, r) = reader_for((0u64..100).collect());
    assert_matches_linear_scan(&r, 200, 300);
    assert!(r.filter_offsets_pk(200, 300).unwrap().is_empty());
}

#[test]
fn sorted_partition_single_row_match_exact_boundaries() {
    let (_f, r) = reader_for((0u64..100).collect());
    // Range start exactly at first row.
    assert_matches_linear_scan(&r, 0, 0);
    assert_eq!(r.filter_offsets_pk(0, 0).unwrap(), vec![0]);
    // Range end exactly at last row.
    assert_matches_linear_scan(&r, 99, 99);
    assert_eq!(r.filter_offsets_pk(99, 99).unwrap(), vec![99]);
}

#[test]
fn sorted_partition_inverted_range_is_empty() {
    let (_f, r) = reader_for((0u64..100).collect());
    assert!(r.filter_offsets_pk(50, 10).unwrap().is_empty());
}

#[test]
fn sorted_partition_with_duplicate_pks_returns_all_matching_offsets() {
    let (_f, r) = reader_for(vec![1, 2, 2, 2, 3, 4]);
    assert_matches_linear_scan(&r, 2, 2);
    assert_eq!(r.filter_offsets_pk(2, 2).unwrap(), vec![1, 2, 3]);
}

#[test]
fn sorted_partition_large_with_tight_range() {
    let n = 10_000u64;
    let (_f, r) = reader_for((0..n).collect());
    assert!(r.is_pk_sorted().unwrap());
    assert_matches_linear_scan(&r, 4_995, 5_005);
    let offsets = r.filter_offsets_pk(4_995, 5_005).unwrap();
    assert_eq!(offsets.len(), 11);
    assert_eq!(offsets[0], 4_995);
    assert_eq!(*offsets.last().unwrap(), 5_005);
}

// ── correctness on an UNSORTED partition (fallback path) ─────────────────────

#[test]
fn unsorted_partition_falls_back_to_linear_scan_and_is_still_correct() {
    let (_f, r) = reader_for(vec![5, 1, 9, 2, 8, 0, 7, 3, 6, 4]);
    assert!(!r.is_pk_sorted().unwrap());
    for (lo, hi) in [(0u64, 3), (4, 4), (10, 20), (0, 9), (9, 0)] {
        assert_matches_linear_scan(&r, lo, hi);
    }
}

#[test]
fn unsorted_partition_partial_overlap_matches_linear_scan() {
    let (_f, r) = reader_for(vec![50, 10, 30, 90, 20, 70, 60, 40, 80, 0]);
    assert_matches_linear_scan(&r, 25, 65);
    let mut expected = r.filter_offsets(0, 25, 65).unwrap();
    let mut actual = r.filter_offsets_pk(25, 65).unwrap();
    expected.sort_unstable();
    actual.sort_unstable();
    assert_eq!(actual, expected);
}
