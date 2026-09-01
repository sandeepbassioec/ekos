//! Layer 78 — Query SIMD (VV-QRY-006..010).
//!
//! Covers: count_scan, sum_scan, min_scan, max_scan, pk_lookup_batch,
//! fk_exists_batch. All aggregate paths verified against scalar oracles.

use std::collections::HashMap;

use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
use vectorvolt_query::{QueryEngine, catalog::TableCatalog};
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

fn build_partition(file_id: u16, partition_id: u16, pks: &[u64], vals: &[u64]) -> PartitionReader {
    let mut p = MicroPartition::new(file_id, partition_id);
    p.push_column("pk", ColumnValues::Raw(pks.to_vec()))
        .unwrap();
    p.push_column("v", ColumnValues::auto_encode(vals.to_vec()))
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp.as_file(), &buf).unwrap();
    PartitionReader::open(tmp.path()).unwrap()
}

fn engine_with(pks: &[u64], vals: &[u64]) -> QueryEngine {
    let reader = build_partition(0, 0, pks, vals);

    let mut pk_index = CuckooMap::with_capacity(pks.len().next_power_of_two().max(8));
    for (offset, &pk) in pks.iter().enumerate() {
        pk_index
            .insert(pk, RowDescriptor::new(0, 0, offset as u32))
            .unwrap();
    }

    let mut fk_filter = CuckooFilter::with_capacity(pks.len().next_power_of_two().max(8));
    for &pk in pks {
        fk_filter.insert(pk).unwrap();
    }

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        fk_filter,
        partitions,
    );
    QueryEngine::new(catalog)
}

// ── count_scan ────────────────────────────────────────────────────────────────

#[test]
fn count_scan_agrees_with_range_scan_len() {
    let pks: Vec<u64> = (1..=100).collect();
    let vals: Vec<u64> = (0..100).collect();
    let engine = engine_with(&pks, &vals);

    for (lo, hi) in [(0u64, 49), (50, 99), (0, 99), (100, 200)] {
        let count = engine.count_scan(1, lo, hi).unwrap();
        let rows = engine.range_scan(1, lo, hi).unwrap();
        assert_eq!(count, rows.len() as u64, "mismatch at [{lo},{hi}]");
    }
}

#[test]
fn count_scan_inverted_range_is_zero() {
    let pks = [1u64, 2, 3];
    let vals = [10u64, 20, 30];
    let engine = engine_with(&pks, &vals);
    assert_eq!(engine.count_scan(1, 100, 50).unwrap(), 0);
}

#[test]
fn count_scan_none_match() {
    let pks = [1u64, 2, 3];
    let vals = [1u64, 2, 3];
    let engine = engine_with(&pks, &vals);
    assert_eq!(engine.count_scan(1, 1000, 2000).unwrap(), 0);
}

// ── sum_scan ──────────────────────────────────────────────────────────────────

#[test]
fn sum_scan_filter_same_col_agrees_with_scalar() {
    let pks: Vec<u64> = (1..=20).collect();
    let vals: Vec<u64> = (0..20).collect();
    let engine = engine_with(&pks, &vals);

    let expected: u64 = (0..20u64).filter(|&v| (5..=14).contains(&v)).sum();
    assert_eq!(engine.sum_scan(1, 5, 14, 1).unwrap(), expected);
}

#[test]
fn sum_scan_sum_pk_col_for_filtered_vals() {
    let pks: Vec<u64> = (10..=19).collect();
    let vals: Vec<u64> = (0..10).collect();
    let engine = engine_with(&pks, &vals);

    // Filter v in [3,6], sum the pk column (col 0).
    let expected: u64 = pks
        .iter()
        .zip(vals.iter())
        .filter(|&(_, &v)| (3..=6).contains(&v))
        .map(|(&pk, _)| pk)
        .sum();
    assert_eq!(engine.sum_scan(1, 3, 6, 0).unwrap(), expected);
}

#[test]
fn sum_scan_empty_filter_returns_zero() {
    let pks = [1u64, 2, 3];
    let vals = [10u64, 20, 30];
    let engine = engine_with(&pks, &vals);
    assert_eq!(engine.sum_scan(1, 500, 999, 1).unwrap(), 0);
}

// ── min_scan / max_scan ───────────────────────────────────────────────────────

#[test]
fn min_scan_returns_correct_min() {
    let pks: Vec<u64> = (1..=10).collect();
    let vals: Vec<u64> = vec![50, 30, 20, 70, 10, 60, 40, 80, 90, 100];
    let engine = engine_with(&pks, &vals);

    // Filter v in [20, 70] — values {50, 30, 20, 70, 60, 40} → min = 20
    let expected = vals
        .iter()
        .filter(|&&v| (20..=70).contains(&v))
        .copied()
        .min();
    assert_eq!(engine.min_scan(1, 20, 70).unwrap(), expected);
}

#[test]
fn max_scan_returns_correct_max() {
    let pks: Vec<u64> = (1..=10).collect();
    let vals: Vec<u64> = vec![50, 30, 20, 70, 10, 60, 40, 80, 90, 100];
    let engine = engine_with(&pks, &vals);

    // Filter v in [20, 70] → max = 70
    let expected = vals
        .iter()
        .filter(|&&v| (20..=70).contains(&v))
        .copied()
        .max();
    assert_eq!(engine.max_scan(1, 20, 70).unwrap(), expected);
}

#[test]
fn min_scan_none_when_no_match() {
    let pks = [1u64, 2, 3];
    let vals = [10u64, 20, 30];
    let engine = engine_with(&pks, &vals);
    assert_eq!(engine.min_scan(1, 100, 200).unwrap(), None);
}

#[test]
fn max_scan_none_when_no_match() {
    let pks = [1u64, 2, 3];
    let vals = [10u64, 20, 30];
    let engine = engine_with(&pks, &vals);
    assert_eq!(engine.max_scan(1, 100, 200).unwrap(), None);
}

// ── pk_lookup_batch ───────────────────────────────────────────────────────────

#[test]
fn pk_lookup_batch_agrees_with_single_lookup() {
    let pks = [1u64, 2, 3, 4, 5];
    let vals = [10u64, 20, 30, 40, 50];
    let engine = engine_with(&pks, &vals);

    let batch = engine.pk_lookup_batch(&[1, 3, 5, 99]).unwrap();
    assert!(batch[0].is_some());
    assert_eq!(batch[0].as_ref().unwrap().values[1], 10);
    assert!(batch[1].is_some());
    assert_eq!(batch[1].as_ref().unwrap().values[1], 30);
    assert!(batch[2].is_some());
    assert_eq!(batch[2].as_ref().unwrap().values[1], 50);
    assert!(batch[3].is_none());
}

#[test]
fn pk_lookup_batch_empty_slice() {
    let engine = engine_with(&[1u64], &[42u64]);
    let result = engine.pk_lookup_batch(&[]).unwrap();
    assert!(result.is_empty());
}

// ── fk_exists_batch ───────────────────────────────────────────────────────────

#[test]
fn fk_exists_batch_agrees_with_single_check() {
    let pks = [10u64, 20, 30];
    let vals = [1u64, 2, 3];
    let engine = engine_with(&pks, &vals);

    let results = engine.fk_exists_batch(&[10, 20, 30, 99]).unwrap();
    assert!(results[0]);
    assert!(results[1]);
    assert!(results[2]);
    assert!(!results[3]);
}

#[test]
fn fk_exists_batch_empty_slice() {
    let engine = engine_with(&[1u64], &[1u64]);
    assert!(engine.fk_exists_batch(&[]).unwrap().is_empty());
}

/// Verify count_scan == range_scan().len() at every row count 0..=20, covering
/// all AVX2 tail-loop remainders (LANES=4 → tail sizes 0–3).
#[test]
fn count_scan_all_lengths_0_to_20_agree() {
    for n in 0usize..=20 {
        let pks: Vec<u64> = (1..=n as u64).collect();
        let vals: Vec<u64> = (0..n as u64).collect();
        let engine = engine_with(&pks, &vals);

        let lo = n as u64 / 4;
        let hi = (n as u64).saturating_sub(1).min(n as u64 * 3 / 4);
        let count = engine.count_scan(1, lo, hi).unwrap();
        let rows = engine.range_scan(1, lo, hi).unwrap();
        assert_eq!(count, rows.len() as u64, "n={n} lo={lo} hi={hi}");
    }
}
