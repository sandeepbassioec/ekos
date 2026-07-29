//! Integration tests for `vectorvolt-storage`.
//!
//! Every test writes a partition to a temp file, reads it back via
//! [`PartitionReader`], and verifies the round-trip.

use std::io::Write;
use tempfile::NamedTempFile;
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_to_temp(partition: &MicroPartition) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp file");
    write_partition(&mut f, partition).expect("write_partition");
    f.flush().expect("flush");
    f
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn raw_column_round_trip() {
    let mut p = MicroPartition::new(1, 0);
    let values: Vec<u64> = (0u64..100).map(|i| i * 7).collect();
    p.push_column("pk", ColumnValues::Raw(values.clone()))
        .unwrap();

    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).expect("open partition");

    assert_eq!(r.file_id(), 1);
    assert_eq!(r.partition_id(), 0);
    assert_eq!(r.row_count(), 100);
    assert_eq!(r.col_count(), 1);
    assert_eq!(r.col_name(0).unwrap(), "pk");

    let col = r.read_column(0).unwrap();
    assert_eq!(col.to_raw_vec(), values);
}

#[test]
fn dict_encoded_column_round_trip() {
    let values: Vec<u64> = (0u64..500).map(|i| i % 7).collect();
    let mut p = MicroPartition::new(0, 3);
    p.push_column("status", ColumnValues::auto_encode(values.clone()))
        .unwrap();

    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();

    let col = r.read_column(0).unwrap();
    assert!(
        matches!(&col, ColumnValues::DictEncoded { dict, .. } if dict.len() == 7),
        "expected DictEncoded with 7 entries"
    );
    assert_eq!(col.to_raw_vec(), values);
}

#[test]
fn multi_column_partition() {
    let n = 200u32;
    let pks: Vec<u64> = (0..n as u64).collect();
    let ages: Vec<u64> = (0..n as u64).map(|i| 18 + i % 60).collect();
    let cats: Vec<u64> = (0..n as u64).map(|i| i % 5).collect();

    let mut p = MicroPartition::new(2, 1);
    p.push_column("pk", ColumnValues::Raw(pks.clone())).unwrap();
    p.push_column("age", ColumnValues::auto_encode(ages.clone()))
        .unwrap();
    p.push_column("category", ColumnValues::auto_encode(cats.clone()))
        .unwrap();

    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();

    assert_eq!(r.col_count(), 3);
    assert_eq!(r.row_count(), n);

    assert_eq!(r.read_column(0).unwrap().to_raw_vec(), pks);
    assert_eq!(r.read_column(1).unwrap().to_raw_vec(), ages);
    assert_eq!(r.read_column(2).unwrap().to_raw_vec(), cats);
}

#[test]
fn zone_min_max_correct() {
    let ages: Vec<u64> = vec![25, 40, 18, 65, 30];
    let mut p = MicroPartition::new(0, 0);
    p.push_column("age", ColumnValues::Raw(ages)).unwrap();

    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();
    let (min, max) = r.zone_min_max(0).unwrap();
    assert_eq!(min, 18);
    assert_eq!(max, 65);
}

#[test]
fn zone_enables_partition_pruning() {
    // Write 4 partitions with non-overlapping age ranges.
    // Then verify that zone_min_max allows pruning partitions outside the query range.
    let ranges = [(20u64, 35u64), (36, 50), (51, 65), (66, 80)];
    let mut files: Vec<NamedTempFile> = Vec::new();

    for (pid, &(lo, hi)) in ranges.iter().enumerate() {
        let mut p = MicroPartition::new(0, pid as u16);
        let ages: Vec<u64> = (lo..=hi).collect();
        p.push_column("age", ColumnValues::Raw(ages)).unwrap();
        files.push(write_to_temp(&p));
    }

    // Query: age BETWEEN 40 AND 60 → should overlap p1 [36,50] and p2 [51,65]
    let (qlo, qhi) = (40u64, 60u64);
    let mut overlapping: Vec<u16> = Vec::new();
    for f in &files {
        let r = PartitionReader::open(f.path()).unwrap();
        let (min, max) = r.zone_min_max(0).unwrap();
        if min <= qhi && max >= qlo {
            overlapping.push(r.partition_id());
        }
    }
    overlapping.sort_unstable();
    assert_eq!(overlapping, vec![1, 2]);
}

#[test]
fn corrupted_file_crc_detected() {
    let mut p = MicroPartition::new(0, 0);
    p.push_column("x", ColumnValues::Raw(vec![1, 2, 3]))
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    // Flip one byte in the data section (byte 100).
    buf[100] ^= 0xFF;

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&buf).unwrap();
    tmp.flush().unwrap();

    let result = PartitionReader::open(tmp.path());
    assert!(
        matches!(
            result,
            Err(vectorvolt_storage::StorageError::FileCrcMismatch { .. })
        ),
        "expected FileCrcMismatch, got {:?}",
        result.err()
    );
}

#[test]
fn corrupted_header_crc_detected() {
    let mut p = MicroPartition::new(0, 0);
    p.push_column("x", ColumnValues::Raw(vec![1, 2, 3]))
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    // Flip a byte in the header CRC itself — the file CRC would fail first.
    // Instead, corrupt a header field (byte 20 = row_count) while patching
    // the file CRC to match the corrupted content.
    buf[20] ^= 0x01; // flip bit in row_count

    // Recompute file CRC for the corrupted content (excluding last 4 bytes).
    let len = buf.len();
    let new_file_crc = crc32fast::hash(&buf[..len - 4]);
    buf[len - 4..].copy_from_slice(&new_file_crc.to_le_bytes());

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&buf).unwrap();
    tmp.flush().unwrap();

    let result = PartitionReader::open(tmp.path());
    assert!(
        matches!(
            result,
            Err(vectorvolt_storage::StorageError::HeaderCrcMismatch { .. })
        ),
        "expected HeaderCrcMismatch, got {:?}",
        result.err()
    );
}

#[test]
fn column_out_of_range_errors() {
    let mut p = MicroPartition::new(0, 0);
    p.push_column("a", ColumnValues::Raw(vec![1, 2])).unwrap();
    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();
    assert!(matches!(
        r.read_column(1),
        Err(vectorvolt_storage::StorageError::ColumnOutOfRange(1, 1))
    ));
}

#[test]
fn file_id_and_partition_id_preserved() {
    let mut p = MicroPartition::new(255, 65535);
    p.push_column("v", ColumnValues::Raw(vec![42])).unwrap();
    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();
    assert_eq!(r.file_id(), 255);
    assert_eq!(r.partition_id(), 65535);
}
