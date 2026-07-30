//! Encoding round-trips through the real `.vvsst` file format
//! (VV-STO-009/010/011).
//!
//! The in-memory encoding tests in `column.rs` prove that encode→decode is the
//! identity in RAM. They say nothing about the file: an encoding can be perfectly
//! correct in memory and still be laid out wrong on disk — a bad `dict_byte_len`,
//! a data offset computed from the wrong length, a bit width that never got
//! persisted. Those bugs appear only after a write/read cycle.

use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

fn write_to_temp(partition: &MicroPartition) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp file");
    write_partition(&mut f, partition).expect("write_partition");
    f.flush().expect("flush");
    f
}

/// Pseudo-random, full-width values — no runs, no narrow range, all distinct.
fn incompressible(n: usize) -> Vec<u64> {
    (0..n as u64)
        .map(|i| {
            let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            x ^= x >> 29;
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^ (x >> 32)
        })
        .collect()
}

/// Value shapes that between them select all five encodings.
fn encoding_shapes() -> Vec<(&'static str, Vec<u64>)> {
    let distinct = [0u64, u64::MAX / 4, u64::MAX / 2, u64::MAX / 4 * 3, u64::MAX];
    vec![
        ("single-row", vec![7]),
        ("all-identical", vec![42; 500]),
        ("sequential", (0..1000).collect()),
        ("sorted-sparse", (0..1000).map(|i| i * 1_000_003).collect()),
        (
            "narrow-range",
            (0..1000).map(|i| 5_000_000 + i % 250).collect(),
        ),
        (
            "few-long-runs",
            (0..1000).map(|i| (i / 250) * 9_999_991).collect(),
        ),
        (
            "wide-low-card",
            (0..1000).map(|i| distinct[(i * 7) % 5]).collect(),
        ),
        ("incompressible", incompressible(1000)),
        ("u64-extremes", vec![0, u64::MAX, 1, u64::MAX - 1]),
        ("block-boundary-plus-one", (0..65u64).collect()),
        ("descending", (0..1000).rev().collect()),
    ]
}

/// Every shape survives a write/read cycle, checked through both read paths.
///
/// `read_value` (the O(1) point path, decoding straight from the mmap) and
/// `read_column` (bulk decode) are separate implementations for every encoding.
/// They can disagree — and a disagreement means a point lookup returns a
/// different value than a scan does for the same row, which is the kind of bug
/// that gets blamed on caching for weeks. So both are checked against the input.
#[test]
fn every_encoding_round_trips_through_a_file() {
    for (name, values) in encoding_shapes() {
        let col = ColumnValues::auto_encode(values.clone());
        let encoding = col.encoding_byte();

        let mut p = MicroPartition::new(3, 11);
        p.push_column("v", col).unwrap();
        let f = write_to_temp(&p);
        let r = PartitionReader::open(f.path()).expect("open");

        assert_eq!(
            r.row_count() as usize,
            values.len(),
            "{name} (encoding {encoding}): row_count"
        );

        let bulk = r.read_column(0).expect("read_column").to_raw_vec();
        assert_eq!(bulk, values, "{name} (encoding {encoding}): read_column");

        for (i, &want) in values.iter().enumerate() {
            assert_eq!(
                r.read_value(0, i).expect("read_value"),
                want,
                "{name} (encoding {encoding}): read_value({i})"
            );
        }

        let (zmin, zmax) = r.zone_min_max(0).expect("zone_min_max");
        assert_eq!(
            (zmin, zmax),
            (
                values.iter().copied().min().unwrap(),
                values.iter().copied().max().unwrap()
            ),
            "{name} (encoding {encoding}): zone"
        );
    }
}

/// All five encodings in ONE file, so a wrong `payload_bytes()` for any of them
/// misaligns every later column's data offset.
///
/// A single-column file hides offset bugs completely: column 0 always starts
/// immediately after the metadata no matter what length it claims. Only a
/// multi-column file where each column has a *different* encoding exercises the
/// cursor arithmetic that the writer and reader must agree on.
#[test]
fn mixed_encodings_in_one_file_keep_their_offsets() {
    let n = 1000usize;
    let distinct = [0u64, u64::MAX / 2, u64::MAX];
    let columns: Vec<(&str, Vec<u64>)> = vec![
        ("incompressible", incompressible(n)),
        (
            "wide_low_card",
            (0..n).map(|i| distinct[(i * 7) % 3]).collect(),
        ),
        (
            "few_runs",
            (0..n as u64).map(|i| (i / 250) * 9_999_991).collect(),
        ),
        (
            "narrow",
            (0..n as u64).map(|i| 8_000_000 + i % 100).collect(),
        ),
        ("sorted", (0..n as u64).map(|i| i * 1_000_003).collect()),
    ];

    let mut p = MicroPartition::new(4, 12);
    let mut encodings = Vec::new();
    for (name, vals) in columns.iter() {
        let col = ColumnValues::auto_encode(vals.clone());
        encodings.push(col.encoding_byte());
        p.push_column(name.to_string(), col).unwrap();
    }

    // Only meaningful if the encodings actually differ. If a future cost-model
    // change collapses them all to one, this fails loudly rather than passing
    // while testing nothing.
    let mut uniq = encodings.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert!(
        uniq.len() >= 4,
        "expected >=4 distinct encodings to exercise offset arithmetic, got {encodings:?}"
    );

    let f = write_to_temp(&p);
    let r = PartitionReader::open(f.path()).expect("open");

    for (c, (name, vals)) in columns.iter().enumerate() {
        assert_eq!(
            r.read_column(c).unwrap().to_raw_vec(),
            *vals,
            "column {c} ({name}, encoding {}): bulk decode",
            encodings[c]
        );
        for (i, &want) in vals.iter().enumerate() {
            assert_eq!(
                r.read_value(c, i).unwrap(),
                want,
                "column {c} ({name}, encoding {}): read_value({i})",
                encodings[c]
            );
        }
    }

    // Whole rows must reassemble across the encoding mix.
    for row in [0usize, 1, 249, 250, 500, 999] {
        let got = r.read_row(row).unwrap();
        let want: Vec<u64> = columns.iter().map(|(_, v)| v[row]).collect();
        assert_eq!(got, want, "read_row({row}) across mixed encodings");
    }
}

/// Range filtering and materialization work on every encoding, not just `Raw`.
///
/// `filter_offsets` decodes the column and hands it to the SIMD kernel, so it is
/// encoding-agnostic by construction — asserted anyway, because "by
/// construction" is exactly how the `read_value` bounds bug got in.
#[test]
fn filter_and_materialize_work_on_every_encoding() {
    for (name, values) in encoding_shapes() {
        if values.len() < 10 {
            continue;
        }
        let col = ColumnValues::auto_encode(values.clone());
        let encoding = col.encoding_byte();
        let mut p = MicroPartition::new(5, 13);
        p.push_column("v", col).unwrap();
        let f = write_to_temp(&p);
        let r = PartitionReader::open(f.path()).expect("open");

        let mut sorted = values.clone();
        sorted.sort_unstable();
        let lo = sorted[sorted.len() / 4];
        let hi = sorted[sorted.len() * 3 / 4];

        let got = r.filter_offsets(0, lo, hi).expect("filter_offsets");
        let want: Vec<u32> = values
            .iter()
            .enumerate()
            .filter(|(_, v)| **v >= lo && **v <= hi)
            .map(|(i, _)| i as u32)
            .collect();
        assert_eq!(got, want, "{name} (encoding {encoding}): filter_offsets");

        let rows = r.materialize_rows(&got).expect("materialize_rows");
        assert_eq!(rows.len(), want.len(), "{name}: materialized row count");
        for (row, &off) in rows.iter().zip(&want) {
            assert_eq!(
                row[0], values[off as usize],
                "{name} (encoding {encoding}): materialized row {off}"
            );
        }
    }
}

/// Compression shrinks the whole file, not just the payload.
///
/// The in-memory test asserts payload ratios; this asserts what a user actually
/// observes. Separate because header, schema, and column metadata are fixed
/// overhead a payload-only ratio ignores.
#[test]
fn compressible_columns_produce_smaller_files() {
    let n = 8192usize;
    let sorted: Vec<u64> = (0..n as u64).collect();

    let mut compressed = MicroPartition::new(6, 14);
    compressed
        .push_column("v", ColumnValues::auto_encode(sorted.clone()))
        .unwrap();
    let cf = write_to_temp(&compressed);
    let compressed_len = std::fs::metadata(cf.path()).unwrap().len();

    let mut raw = MicroPartition::new(6, 15);
    raw.push_column("v", ColumnValues::Raw(sorted.clone()))
        .unwrap();
    let rf = write_to_temp(&raw);
    let raw_len = std::fs::metadata(rf.path()).unwrap().len();

    assert!(
        compressed_len * 4 < raw_len,
        "sorted column: {compressed_len} B compressed vs {raw_len} B raw — expected >4x"
    );

    let r = PartitionReader::open(cf.path()).unwrap();
    assert_eq!(r.read_column(0).unwrap().to_raw_vec(), sorted);
}

/// A corrupt `bit_width` must produce an error, never a panic or a wrong value.
///
/// `bit_width` is one byte read straight out of the file and then used as a shift
/// distance. In Rust a shift of 64 or more panics in debug and is masked in
/// release — either way it must never be reached with an unvalidated value, which
/// makes this the most dangerous byte in the format.
#[test]
fn corrupt_bit_width_is_rejected_not_panicked_on() {
    let values: Vec<u64> = (0..500).collect();
    let col = ColumnValues::auto_encode(values.clone());
    assert!(
        col.bit_width() > 0,
        "expected a bit-packed encoding, got {}",
        col.encoding_byte()
    );

    let mut p = MicroPartition::new(7, 16);
    p.push_column("v", col).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    // Column metadata starts after the 32-byte header and the schema section.
    // Schema for one column named "v" is u16 len + 1 byte = 3 bytes. Byte 1 of
    // the metadata entry is `bit_width`.
    let bit_width_pos = 32 + 3 + 1;

    for bad in [65u8, 128, 255] {
        let mut corrupt = buf.clone();
        corrupt[bit_width_pos] = bad;
        // Re-CRC so the file passes its integrity gate and the bad width is
        // actually reached — otherwise the CRC check masks what is being tested.
        let len = corrupt.len();
        let crc = crc32fast::hash(&corrupt[..len - 4]);
        corrupt[len - 4..].copy_from_slice(&crc.to_le_bytes());

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.vvsst");
        std::fs::write(&path, &corrupt).unwrap();

        let r = PartitionReader::open(&path).expect("header should still parse");
        assert!(
            r.read_value(0, 0).is_err(),
            "bit_width {bad}: read_value should error, not panic or fabricate"
        );
        assert!(
            r.read_column(0).is_err(),
            "bit_width {bad}: read_column should error"
        );
    }
}

/// A truncated file must be rejected at `open`, by the file CRC.
///
/// ## Which defence catches truncation, and which does not
///
/// My first version of this test cut the file short and then **recomputed the
/// file CRC** so the read path's bounds checks would be the thing under test. It
/// failed — and the code was right. Truncating the payload while re-CRCing
/// produces a file whose every computed offset still lies inside the mapping (the
/// 4 CRC bytes land exactly where the lost payload bytes were), so a bounds check
/// cannot possibly notice. It read a value composed partly of CRC bytes and
/// returned it.
///
/// The lesson: **bounds checks catch out-of-range offsets; the CRC catches
/// truncation.** They are not interchangeable, and a test that recomputes the CRC
/// has disabled the only defence that applies. So this test leaves the CRC alone
/// — which is also what a real truncated file looks like, since nothing helpfully
/// re-CRCs a partial write.
#[test]
fn truncated_file_is_rejected() {
    let values: Vec<u64> = (0..500).collect();
    let mut p = MicroPartition::new(8, 17);
    p.push_column("v", ColumnValues::auto_encode(values))
        .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    for keep in [
        1usize,
        8,
        33,
        40,
        60,
        buf.len() / 2,
        buf.len() - 8,
        buf.len() - 1,
    ] {
        if keep >= buf.len() {
            continue;
        }
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("short.vvsst");
        std::fs::write(&path, &buf[..keep]).unwrap();

        assert!(
            PartitionReader::open(&path).is_err(),
            "a file truncated to {keep} of {} bytes was accepted",
            buf.len()
        );
    }
}

/// A corrupt *offset* must error rather than read past the mapping.
///
/// This is the case bounds checks exist for, and it needs the CRC recomputed so
/// the bad offset is actually reached. Distinct from truncation above: the file is
/// full length and internally consistent by checksum, but a metadata field points
/// somewhere it should not — a bit flip in exactly the wrong place, or a
/// deliberately malformed file.
#[test]
fn corrupt_data_offset_is_rejected_not_panicked_on() {
    let values: Vec<u64> = (0..500).collect();
    let mut p = MicroPartition::new(9, 18);
    p.push_column("v", ColumnValues::auto_encode(values))
        .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    write_partition(&mut buf, &p).unwrap();

    // Column metadata entry starts at 32 (header) + 3 (schema: u16 len + "v").
    // Layout: encoding[1] bit_width[1] pad[6] data_offset[8] data_byte_len[8]
    //         dict_offset[8] dict_byte_len[8] zone_min[8] zone_max[8]
    let meta = 32 + 3;
    let data_offset_pos = meta + 8;
    // Both offsets are corrupted, because which one a read uses depends on the
    // encoding: single-section encodings (Raw, Frame) read from `data_offset`,
    // two-section ones (Dict, Rle, BlockedFrame) from `dict_offset`. Corrupting
    // only `data_offset` silently tests nothing for the latter — which is how the
    // first version of this test passed while proving nothing.
    let dict_offset_pos = meta + 24;

    for bad in [u64::MAX, u64::MAX - 7, buf.len() as u64, 1u64 << 60] {
        let mut corrupt = buf.clone();
        corrupt[data_offset_pos..data_offset_pos + 8].copy_from_slice(&bad.to_le_bytes());
        corrupt[dict_offset_pos..dict_offset_pos + 8].copy_from_slice(&bad.to_le_bytes());
        let len = corrupt.len();
        let crc = crc32fast::hash(&corrupt[..len - 4]);
        corrupt[len - 4..].copy_from_slice(&crc.to_le_bytes());

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("badoff.vvsst");
        std::fs::write(&path, &corrupt).unwrap();

        // `open` may succeed — the header and metadata are structurally fine. The
        // reads must not panic, and must not fabricate a value.
        if let Ok(r) = PartitionReader::open(&path) {
            assert!(
                r.read_value(0, 0).is_err(),
                "data_offset {bad}: read_value should error"
            );
            assert!(
                r.read_column(0).is_err(),
                "data_offset {bad}: read_column should error"
            );
        }
    }
}

/// Reports the compression ratio each shape achieves. Not an assertion — the
/// numbers quoted in the commit message come from here, so they are reproducible.
#[test]
fn report_compression_ratios() {
    let n = 8192usize;
    let distinct = [0u64, u64::MAX / 2, u64::MAX];
    let cases: Vec<(&str, Vec<u64>)> = vec![
        ("sequential PK", (0..n as u64).collect()),
        (
            "timestamps (ms, drifting)",
            (0..n as u64).map(|i| 1_760_000_000_000 + i * 37).collect(),
        ),
        (
            "narrow range (0..1000)",
            (0..n as u64).map(|i| 10_000_000_000 + i % 1000).collect(),
        ),
        (
            "status column (7 values)",
            (0..n as u64).map(|i| i % 7).collect(),
        ),
        (
            "few long runs",
            (0..n as u64).map(|i| (i / 1024) * 9_999_991).collect(),
        ),
        (
            "wide low cardinality",
            (0..n).map(|i| distinct[(i * 7) % 3]).collect(),
        ),
        ("incompressible", incompressible(n)),
    ];
    println!(
        "\n  {:<28} {:>6} {:>10} {:>8}  vs raw {} B",
        "shape",
        "enc",
        "bytes",
        "ratio",
        n * 8
    );
    for (name, values) in cases {
        let col = ColumnValues::auto_encode(values.clone());
        let bytes = col.payload_bytes();
        let ratio = bytes as f64 / (n * 8) as f64;
        println!(
            "  {:<28} {:>6} {:>10} {:>7.3}x  {:.1}x smaller",
            name,
            col.encoding_byte(),
            bytes,
            ratio,
            1.0 / ratio
        );
        assert_eq!(col.to_raw_vec(), values, "{name}");
    }
}
