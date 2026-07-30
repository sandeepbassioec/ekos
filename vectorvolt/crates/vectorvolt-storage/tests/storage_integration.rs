//! Integration tests for `vectorvolt-storage`.
//!
//! Every test writes a partition to a temp file, reads it back via
//! [`PartitionReader`], and verifies the round-trip.

use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use vectorvolt_storage::{
    ColumnValues, MicroPartition, PartitionReader, StorageError, write_partition,
};

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

/// A low-cardinality column round-trips — and is no longer dictionary-encoded.
///
/// This test used to assert `DictEncoded`. Once VV-STO-010/011 landed, values
/// `0..=6` are better served by a frame encoding: they span a range of 6, so 3
/// bits each is 200 B against the dictionary's `7·8 + 500·4 = 2056 B`. The
/// dictionary now wins only when the distinct values are also *far apart*.
///
/// The assertion was narrowed to what actually matters here — the round-trip —
/// plus a check that whatever was chosen beats a dictionary, so this still fails
/// if the cost model regresses.
#[test]
fn low_cardinality_column_round_trip() {
    let values: Vec<u64> = (0u64..500).map(|i| i % 7).collect();
    let mut p = MicroPartition::new(0, 3);
    p.push_column("status", ColumnValues::auto_encode(values.clone()))
        .unwrap();

    let tmp = write_to_temp(&p);
    let r = PartitionReader::open(tmp.path()).unwrap();

    let col = r.read_column(0).unwrap();
    assert_eq!(col.to_raw_vec(), values);
    assert!(
        col.payload_bytes() < 7 * 8 + 500 * 4,
        "encoding {} used {} B, worse than a dictionary would",
        col.encoding_byte(),
        col.payload_bytes()
    );
    // Point reads agree with the bulk decode.
    for (i, &want) in values.iter().enumerate() {
        assert_eq!(r.read_value(0, i).unwrap(), want, "read_value({i})");
    }
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

// ── read_value robustness (added with the point-lookup consolidation) ─────────
//
// `read_value` computes byte offsets from column metadata and, for dictionary
// columns, from an index stored *in the file*. None of that may be trusted. The
// first implementation used direct slice indexing (`data[a..b]`), which **panics**
// on an out-of-range range — so a stale RowDescriptor or a corrupt dictionary
// index would abort the process instead of returning an error. These tests pin the
// error behaviour so that cannot regress.

#[test]
fn read_value_rejects_row_offset_past_the_end() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bounds.vvsst");

    let mut mp = MicroPartition::new(0, 0);
    mp.push_column("id", ColumnValues::Raw(vec![10, 20, 30]))
        .unwrap();
    let mut f = std::fs::File::create(&path).unwrap();
    write_partition(&mut f, &mp).unwrap();
    drop(f);

    let reader = PartitionReader::open(&path).unwrap();
    assert_eq!(reader.row_count(), 3);

    // Valid offsets still work.
    assert_eq!(reader.read_value(0, 0).unwrap(), 10);
    assert_eq!(reader.read_value(0, 2).unwrap(), 30);

    // One past the end — the case a RowDescriptor cached across a compaction
    // produces. Must be an error, and must NOT panic.
    match reader.read_value(0, 3) {
        Err(StorageError::RowOffsetOutOfBounds {
            row_offset,
            row_count,
        }) => {
            assert_eq!(row_offset, 3);
            assert_eq!(row_count, 3);
        }
        Err(other) => panic!("wrong error for out-of-range offset: {other}"),
        Ok(v) => panic!("read_value returned {v} for a nonexistent row"),
    }

    // Wildly out of range must behave the same way, not wrap or panic.
    assert!(matches!(
        reader.read_value(0, usize::MAX / 2),
        Err(StorageError::RowOffsetOutOfBounds { .. })
    ));
}

#[test]
fn read_value_rejects_unknown_column() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cols.vvsst");
    let mut mp = MicroPartition::new(0, 0);
    mp.push_column("id", ColumnValues::Raw(vec![1, 2])).unwrap();
    let mut f = std::fs::File::create(&path).unwrap();
    write_partition(&mut f, &mp).unwrap();
    drop(f);

    let reader = PartitionReader::open(&path).unwrap();
    assert!(matches!(
        reader.read_value(5, 0),
        Err(StorageError::ColumnOutOfRange(5, 1))
    ));
}

#[test]
fn read_value_and_read_column_agree_on_every_row_and_encoding() {
    // The consolidation only holds if the two paths return identical values.
    // Dictionary-encoded columns exercise the two-step index-then-value read.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("agree.vvsst");

    let ids: Vec<u64> = (0..200u64).collect();
    // Few distinct values, so `auto_encode` picks dictionary encoding.
    let repeated: Vec<u64> = (0..200u64).map(|i| (i % 5) * 1_000_000).collect();

    let mut mp = MicroPartition::new(0, 0);
    mp.push_column("id", ColumnValues::Raw(ids.clone()))
        .unwrap();
    mp.push_column("grade", ColumnValues::auto_encode(repeated.clone()))
        .unwrap();
    let mut f = std::fs::File::create(&path).unwrap();
    write_partition(&mut f, &mp).unwrap();
    drop(f);

    let reader = PartitionReader::open(&path).unwrap();
    for col in 0..reader.col_count() as usize {
        let whole = reader.read_column(col).unwrap().to_raw_vec();
        assert_eq!(whole.len(), 200, "column {col} has the wrong row count");
        for (row, expected) in whole.iter().enumerate() {
            assert_eq!(
                reader.read_value(col, row).unwrap(),
                *expected,
                "read_value and read_column disagree at col {col} row {row}"
            );
        }
    }
    // And against the source data, so both paths being wrong the same way fails.
    for row in 0..200usize {
        assert_eq!(reader.read_value(0, row).unwrap(), ids[row]);
        assert_eq!(reader.read_value(1, row).unwrap(), repeated[row]);
    }
}
