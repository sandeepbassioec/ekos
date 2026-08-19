//! Layer 77 — Storage SIMD scan (VV-STO-009..012).
//!
//! Covers: count_matching, column_sum, column_min_max, and SIMD min_max in
//! zone_min_max (column.rs). All paths verified against scalar oracles.

use std::io::Write;
use tempfile::NamedTempFile;
use vectorvolt_storage::{
    ColumnValues, MicroPartition, PartitionReader, WriteOptions, write_partition,
    write_partition_with,
};

fn write_partition_to_file(p: &MicroPartition, opts: Option<WriteOptions>) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    match opts {
        Some(o) => write_partition_with(&mut f, p, o).unwrap(),
        None => write_partition(&mut f, p).unwrap(),
    }
    f.flush().unwrap();
    f
}

fn reader(p: &MicroPartition) -> (NamedTempFile, PartitionReader) {
    let f = write_partition_to_file(p, None);
    let r = PartitionReader::open(f.path()).unwrap();
    (f, r)
}

// ── count_matching tests ───────────────────────────────────────────────────────

#[test]
fn count_matching_agrees_with_filter_offsets() {
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode((0u64..1000).collect()))
        .unwrap();
    let (_f, r) = reader(&p);

    for (lo, hi) in [(0u64, 100), (500, 999), (0, 0), (999, 999), (0, u64::MAX)] {
        let count = r.count_matching(0, lo, hi).unwrap();
        let offsets = r.filter_offsets(0, lo, hi).unwrap();
        assert_eq!(
            count,
            offsets.len(),
            "count_matching disagrees with filter_offsets for [{lo},{hi}]"
        );
    }
}

#[test]
fn count_matching_inverted_range_is_zero() {
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vec![10u64, 20, 30]))
        .unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.count_matching(0, 100, 50).unwrap(), 0);
}

#[test]
fn count_matching_all_match() {
    let vals: Vec<u64> = (0..256).collect();
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals.clone()))
        .unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.count_matching(0, 0, 255).unwrap(), 256);
}

#[test]
fn count_matching_none_match() {
    let vals: Vec<u64> = (0..100).collect();
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.count_matching(0, 500, 999).unwrap(), 0);
}

#[test]
fn count_matching_on_compressed_column() {
    let vals: Vec<u64> = vec![42u64; 1000];
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let f = write_partition_to_file(&p, Some(WriteOptions::default().with_lz4()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert_eq!(r.count_matching(0, 42, 42).unwrap(), 1000);
    assert_eq!(r.count_matching(0, 43, 43).unwrap(), 0);
}

// ── column_sum tests ────────────────────────────────────────────────────────────

#[test]
fn column_sum_matches_scalar_sum() {
    let vals: Vec<u64> = (1u64..=100).collect();
    let expected: u64 = vals.iter().sum();
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.column_sum(0).unwrap(), expected);
}

#[test]
fn column_sum_empty_partition() {
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vec![]))
        .unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.column_sum(0).unwrap(), 0);
}

#[test]
fn column_sum_wraps_on_overflow() {
    let vals = vec![u64::MAX, 1u64];
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.column_sum(0).unwrap(), 0u64); // MAX.wrapping_add(1) = 0
}

#[test]
fn column_sum_on_compressed_partition() {
    let vals: Vec<u64> = (0..500).collect();
    let expected: u64 = vals.iter().sum();
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let f = write_partition_to_file(&p, Some(WriteOptions::default().with_zstd()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert_eq!(r.column_sum(0).unwrap(), expected);
}

// ── column_min_max tests ─────────────────────────────────────────────────────────

#[test]
fn column_min_max_agrees_with_zone_min_max() {
    let vals: Vec<u64> = (100u64..=200).collect();
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let (_f, r) = reader(&p);
    let (zone_lo, zone_hi) = r.zone_min_max(0).unwrap();
    let computed = r.column_min_max(0).unwrap().unwrap();
    assert_eq!(computed, (zone_lo, zone_hi));
}

#[test]
fn column_min_max_none_on_empty() {
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vec![]))
        .unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.column_min_max(0).unwrap(), None);
}

#[test]
fn column_min_max_handles_u64_max() {
    let vals = vec![0u64, u64::MAX, u64::MAX / 2];
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let (_f, r) = reader(&p);
    assert_eq!(r.column_min_max(0).unwrap(), Some((0, u64::MAX)));
}

#[test]
fn column_min_max_on_zstd_partition() {
    let vals: Vec<u64> = (1000u64..=1500).collect();
    let expected = (1000u64, 1500u64);
    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
    let f = write_partition_to_file(&p, Some(WriteOptions::default().with_zstd()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert_eq!(r.column_min_max(0).unwrap(), Some(expected));
}

/// Verify count_matching == filter_offsets at every row count 0..=20, covering
/// all AVX2 tail-loop remainders (LANES=4, so 0–3 are the tail sizes).
#[test]
fn count_matching_all_lengths_0_to_20_agree() {
    for n in 0usize..=20 {
        let vals: Vec<u64> = (0..n as u64).collect();
        let mut p = MicroPartition::new(1, 1);
        p.push_column("v", ColumnValues::auto_encode(vals)).unwrap();
        let (_f, r) = reader(&p);
        let lo = n as u64 / 4;
        let hi = (n as u64).saturating_sub(1).min(n as u64 * 3 / 4);
        let count = r.count_matching(0, lo, hi).unwrap();
        let offsets = r.filter_offsets(0, lo, hi).unwrap();
        assert_eq!(count, offsets.len(), "disagreement at n={n}");
    }
}

// ── zone_min_max SIMD (column.rs) ────────────────────────────────────────────────

#[test]
fn zone_min_max_in_writer_uses_simd_result() {
    // After write, the stored zone stats must match simd::min_max over the raw values.
    let vals: Vec<u64> = vec![42u64, 1, u64::MAX / 3, 9999, 0];
    let expected_min = *vals.iter().min().unwrap();
    let expected_max = *vals.iter().max().unwrap();

    let mut p = MicroPartition::new(1, 1);
    p.push_column("v", ColumnValues::Raw(vals)).unwrap();
    let (_f, r) = reader(&p);
    let (zmin, zmax) = r.zone_min_max(0).unwrap();
    assert_eq!(zmin, expected_min);
    assert_eq!(zmax, expected_max);
}
