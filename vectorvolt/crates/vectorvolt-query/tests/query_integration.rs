//! Integration tests for `vectorvolt-query`.
//!
//! Each test builds a full TableCatalog with real PartitionReaders (written to
//! temp files) and exercises the QueryEngine end-to-end.

use std::collections::HashMap;
use std::io::Write;

use tempfile::NamedTempFile;
use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
use vectorvolt_query::{QueryEngine, TableCatalog};
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

// ── helpers ───────────────────────────────────────────────────────────────────

struct TempPartition {
    _file: NamedTempFile,
    reader: PartitionReader,
}

fn make_partition(file_id: u16, partition_id: u16, pks: &[u64], values: &[u64]) -> TempPartition {
    let mut p = MicroPartition::new(file_id, partition_id);
    p.push_column("pk", ColumnValues::Raw(pks.to_vec()))
        .unwrap();
    p.push_column("val", ColumnValues::auto_encode(values.to_vec()))
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&buf).unwrap();
    tmp.flush().unwrap();

    let reader = PartitionReader::open(tmp.path()).unwrap();
    TempPartition { _file: tmp, reader }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// PK point lookup returns the correct row.
#[test]
fn pk_lookup_finds_correct_row() {
    let tp = make_partition(0, 0, &[100u64, 200, 300], &[10u64, 20, 30]);

    let mut pk_index = CuckooMap::with_capacity(8);
    pk_index.insert(100, RowDescriptor::new(0, 0, 0)).unwrap();
    pk_index.insert(200, RowDescriptor::new(0, 0, 1)).unwrap();
    pk_index.insert(300, RowDescriptor::new(0, 0, 2)).unwrap();

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp.reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        CuckooFilter::with_capacity(8),
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    let row = engine.pk_lookup(200).unwrap().expect("pk=200 must exist");
    assert_eq!(row.pk, 200);
    assert_eq!(row.values[0], 200); // col 0 = pk
    assert_eq!(row.values[1], 20); // col 1 = val
}

/// PK lookup for a missing key returns None.
#[test]
fn pk_lookup_missing_key_returns_none() {
    let tp = make_partition(0, 0, &[1u64], &[99u64]);

    let mut pk_index = CuckooMap::with_capacity(8);
    pk_index.insert(1, RowDescriptor::new(0, 0, 0)).unwrap();

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp.reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        CuckooFilter::with_capacity(8),
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    assert!(engine.pk_lookup(42).unwrap().is_none());
}

/// Range scan across two partitions with ZoneMap pruning.
#[test]
fn range_scan_multi_partition_with_zone_pruning() {
    // Partition 0: pks=[1,2,3], val=[10,20,30] → zone [10,30]
    // Partition 1: pks=[4,5,6], val=[40,50,60] → zone [40,60]
    // Query: val BETWEEN 10 AND 30 → only partition 0 should be scanned
    let tp0 = make_partition(0, 0, &[1u64, 2, 3], &[10u64, 20, 30]);
    let tp1 = make_partition(0, 1, &[4u64, 5, 6], &[40u64, 50, 60]);

    let mut pk_index = CuckooMap::with_capacity(16);
    pk_index.insert(1, RowDescriptor::new(0, 0, 0)).unwrap();
    pk_index.insert(2, RowDescriptor::new(0, 0, 1)).unwrap();
    pk_index.insert(3, RowDescriptor::new(0, 0, 2)).unwrap();
    pk_index.insert(4, RowDescriptor::new(0, 1, 0)).unwrap();
    pk_index.insert(5, RowDescriptor::new(0, 1, 1)).unwrap();
    pk_index.insert(6, RowDescriptor::new(0, 1, 2)).unwrap();

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp0.reader);
    partitions.insert((0u16, 1u16), tp1.reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        CuckooFilter::with_capacity(16),
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    let mut rows = engine.range_scan(1, 10, 30).unwrap();
    rows.sort_by_key(|r| r.values[1]);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].values[1], 10);
    assert_eq!(rows[1].values[1], 20);
    assert_eq!(rows[2].values[1], 30);

    // No rows from partition 1 (vals 40-60 outside [10,30])
    for r in &rows {
        assert!(r.values[1] <= 30);
    }
}

/// Range scan returns empty slice when no values fall in range.
#[test]
fn range_scan_no_match_returns_empty() {
    let tp = make_partition(0, 0, &[1u64, 2], &[5u64, 10]);

    let mut pk_index = CuckooMap::with_capacity(8);
    pk_index.insert(1, RowDescriptor::new(0, 0, 0)).unwrap();
    pk_index.insert(2, RowDescriptor::new(0, 0, 1)).unwrap();

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp.reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        CuckooFilter::with_capacity(8),
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    let rows = engine.range_scan(1, 100, 200).unwrap();
    assert!(rows.is_empty());
}

/// Secondary index scan resolves PKs to full rows.
#[test]
fn secondary_scan_resolves_rows() {
    let pks = [1u64, 2, 3];
    let vals = [42u64, 99, 42]; // val=42 for pk=1 and pk=3

    let tp = make_partition(0, 0, &pks, &vals);

    let mut pk_index = CuckooMap::with_capacity(8);
    for (offset, &pk) in pks.iter().enumerate() {
        pk_index
            .insert(pk, RowDescriptor::new(0, 0, offset as u32))
            .unwrap();
    }

    // secondary_index: hash(42) → [1, 3], hash(99) → [2]
    let mut secondary_index = SecondaryIndex::new();
    secondary_index.insert(42u32, 1u32); // pk=1 has val=42
    secondary_index.insert(42u32, 3u32); // pk=3 has val=42
    secondary_index.insert(99u32, 2u32); // pk=2 has val=99

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp.reader);

    let catalog = TableCatalog::new(
        pk_index,
        secondary_index,
        ZoneMap::new(),
        CuckooFilter::with_capacity(8),
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    let mut rows = engine.secondary_scan(42u32).unwrap();
    rows.sort_by_key(|r| r.pk);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pk, 1);
    assert_eq!(rows[0].values[1], 42);
    assert_eq!(rows[1].pk, 3);
    assert_eq!(rows[1].values[1], 42);
}

/// FK existence check: inserted keys are found; non-inserted keys are not.
#[test]
fn fk_exists_inserted_keys() {
    let tp = make_partition(0, 0, &[10u64], &[1u64]);

    let mut pk_index = CuckooMap::with_capacity(8);
    pk_index.insert(10, RowDescriptor::new(0, 0, 0)).unwrap();

    let mut fk_filter = CuckooFilter::with_capacity(32);
    for k in [10u64, 20, 30, 40, 50] {
        fk_filter.insert(k).unwrap();
    }

    let mut partitions = HashMap::new();
    partitions.insert((0u16, 0u16), tp.reader);

    let catalog = TableCatalog::new(
        pk_index,
        SecondaryIndex::new(),
        ZoneMap::new(),
        fk_filter,
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    assert!(engine.fk_exists(10));
    assert!(engine.fk_exists(20));
    assert!(engine.fk_exists(50));
    // 9999 was never inserted — very unlikely to be a false positive at this size
    assert!(!engine.fk_exists(9999));
}

/// Full workflow: build catalog, run all 4 query types against one table.
#[test]
fn full_query_workflow() {
    // 6 rows across 2 partitions
    let tp0 = make_partition(1, 0, &[1u64, 2, 3], &[10u64, 20, 30]);
    let tp1 = make_partition(1, 1, &[4u64, 5, 6], &[40u64, 20, 60]);

    let mut pk_index = CuckooMap::with_capacity(16);
    pk_index.insert(1, RowDescriptor::new(1, 0, 0)).unwrap();
    pk_index.insert(2, RowDescriptor::new(1, 0, 1)).unwrap();
    pk_index.insert(3, RowDescriptor::new(1, 0, 2)).unwrap();
    pk_index.insert(4, RowDescriptor::new(1, 1, 0)).unwrap();
    pk_index.insert(5, RowDescriptor::new(1, 1, 1)).unwrap();
    pk_index.insert(6, RowDescriptor::new(1, 1, 2)).unwrap();

    // Secondary: val=20 appears at pk=2 and pk=5
    let mut secondary_index = SecondaryIndex::new();
    secondary_index.insert(20u32, 2u32);
    secondary_index.insert(20u32, 5u32);

    // FK filter covers all PKs
    let mut fk_filter = CuckooFilter::with_capacity(16);
    for pk in 1u64..=6 {
        fk_filter.insert(pk).unwrap();
    }

    let mut partitions = HashMap::new();
    partitions.insert((1u16, 0u16), tp0.reader);
    partitions.insert((1u16, 1u16), tp1.reader);

    let catalog = TableCatalog::new(
        pk_index,
        secondary_index,
        ZoneMap::new(),
        fk_filter,
        partitions,
    );
    let engine = QueryEngine::new(catalog);

    // 1. PK lookup
    assert_eq!(engine.pk_lookup(4).unwrap().unwrap().values[1], 40);
    assert!(engine.pk_lookup(99).unwrap().is_none());

    // 2. Range scan: val BETWEEN 20 AND 40
    let mut range = engine.range_scan(1, 20, 40).unwrap();
    range.sort_by_key(|r| r.pk);
    let range_pks: Vec<u64> = range.iter().map(|r| r.pk).collect();
    assert!(range_pks.contains(&2)); // val=20
    assert!(range_pks.contains(&4)); // val=40
    assert!(range_pks.contains(&5)); // val=20

    // 3. Secondary scan
    let mut sec = engine.secondary_scan(20u32).unwrap();
    sec.sort_by_key(|r| r.pk);
    assert_eq!(sec.len(), 2);
    assert_eq!(sec[0].pk, 2);
    assert_eq!(sec[1].pk, 5);

    // 4. FK exists
    assert!(engine.fk_exists(3));
    assert!(!engine.fk_exists(777));
}
