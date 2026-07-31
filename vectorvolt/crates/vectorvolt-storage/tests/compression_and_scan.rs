//! LZ4 compression and the scan decode path (VV-STO-012/013/014).

use std::io::Write;
use tempfile::NamedTempFile;
use vectorvolt_storage::{
    ColumnValues, MicroPartition, PartitionReader, WriteOptions, write_partition,
    write_partition_with,
};

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

fn shapes(n: usize) -> Vec<(&'static str, Vec<u64>)> {
    let distinct = [0u64, u64::MAX / 4, u64::MAX / 2, u64::MAX];
    vec![
        ("sequential", (0..n as u64).collect()),
        (
            "narrow",
            (0..n as u64).map(|i| 9_000_000 + i % 500).collect(),
        ),
        (
            "runs",
            (0..n as u64).map(|i| (i / 256) * 7_777_777).collect(),
        ),
        (
            "wide-low-card",
            (0..n).map(|i| distinct[(i * 7) % 4]).collect(),
        ),
        ("incompressible", incompressible(n)),
        ("all-identical", vec![99; n]),
    ]
}

fn write_with(partition: &MicroPartition, opts: Option<WriteOptions>) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    match opts {
        Some(o) => write_partition_with(&mut f, partition, o).unwrap(),
        None => write_partition(&mut f, partition).unwrap(),
    }
    f.flush().unwrap();
    f
}

/// LZ4 round-trips losslessly for every encoding.
///
/// Compression sits on top of the encodings, so it multiplies the number of
/// (encoding, compression) pairs rather than adding one case. Each pair decodes
/// through both read paths.
#[test]
fn lz4_round_trips_every_encoding() {
    for (name, values) in shapes(2000) {
        let mut p = MicroPartition::new(1, 1);
        p.push_column("v", ColumnValues::auto_encode(values.clone()))
            .unwrap();
        let f = write_with(&p, Some(WriteOptions::new().with_lz4()));
        let r = PartitionReader::open(f.path()).unwrap();

        let mut buf = Vec::new();
        r.decode_column_into(0, &mut buf).unwrap();
        assert_eq!(buf, values, "{name}: decode_column_into under LZ4");
        assert_eq!(
            r.read_column(0).unwrap().to_raw_vec(),
            values,
            "{name}: read_column under LZ4"
        );
        for i in [0usize, 1, values.len() / 2, values.len() - 1] {
            assert_eq!(
                r.read_value(0, i).unwrap(),
                values[i],
                "{name}: read_value({i})"
            );
        }
        assert_eq!(
            r.zone_min_max(0).unwrap(),
            (
                values.iter().copied().min().unwrap(),
                values.iter().copied().max().unwrap()
            ),
            "{name}: zone survives compression"
        );
    }
}

/// Enabling compression must never make a file bigger.
///
/// LZ4 on already-incompressible bytes adds framing overhead. The writer stores
/// whichever is smaller and tags the column accordingly, so `with_lz4()` is safe
/// to enable blindly on a mixed partition — asserted here because the fallback is
/// easy to write and easy to get subtly wrong (tagging the column compressed
/// while storing verbatim bytes would corrupt every read).
#[test]
fn lz4_never_inflates_a_file() {
    for (name, values) in shapes(4096) {
        let mut p = MicroPartition::new(2, 2);
        p.push_column("v", ColumnValues::auto_encode(values.clone()))
            .unwrap();

        let plain = write_with(&p, None);
        let packed = write_with(&p, Some(WriteOptions::new().with_lz4()));
        let plain_len = std::fs::metadata(plain.path()).unwrap().len();
        let packed_len = std::fs::metadata(packed.path()).unwrap().len();

        assert!(
            packed_len <= plain_len,
            "{name}: LZ4 grew the file, {plain_len} -> {packed_len}"
        );
        // And the fallback still reads correctly.
        let r = PartitionReader::open(packed.path()).unwrap();
        let mut buf = Vec::new();
        r.decode_column_into(0, &mut buf).unwrap();
        assert_eq!(buf, values, "{name}: fallback column decoded wrong");
    }
}

/// `is_point_read_optimal` tells the truth about the latency class.
///
/// The flag exists so a caller can tell that point reads on this partition are
/// O(column) rather than discovering it as unexplained latency. It must report
/// what was STORED, not what was requested — an incompressible column asked to
/// compress stays verbatim and stays O(1).
#[test]
fn point_read_optimality_reflects_what_was_stored() {
    // Compressible → actually compressed → not point-optimal.
    let mut p = MicroPartition::new(3, 3);
    p.push_column(
        "v",
        ColumnValues::Raw((0..4096u64).map(|i| i % 4).collect()),
    )
    .unwrap();
    let f = write_with(&p, Some(WriteOptions::new().with_lz4()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert!(
        r.is_column_compressed(0),
        "compressible column should compress"
    );
    assert!(!r.is_point_read_optimal());

    // Uncompressed write → always point-optimal.
    let f = write_with(&p, None);
    let r = PartitionReader::open(f.path()).unwrap();
    assert!(!r.is_column_compressed(0));
    assert!(r.is_point_read_optimal());

    // Incompressible + LZ4 requested → stored verbatim → still point-optimal.
    let mut p = MicroPartition::new(3, 4);
    p.push_column("v", ColumnValues::Raw(incompressible(4096)))
        .unwrap();
    let f = write_with(&p, Some(WriteOptions::new().with_lz4()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert!(
        !r.is_column_compressed(0),
        "incompressible column must fall back to verbatim"
    );
    assert!(
        r.is_point_read_optimal(),
        "a fallback column must not be reported as slow"
    );
}

/// Mixed compressed and verbatim columns in one file keep their offsets.
///
/// Compression changes each column's stored length, so the writer computes data
/// offsets from serialized bytes rather than from `payload_bytes()`. A single
/// column cannot expose an error there.
#[test]
fn mixed_compressibility_keeps_offsets() {
    let n = 2000usize;
    let cols = shapes(n);
    let mut p = MicroPartition::new(4, 5);
    for (name, vals) in &cols {
        p.push_column(name.to_string(), ColumnValues::auto_encode(vals.clone()))
            .unwrap();
    }
    let f = write_with(&p, Some(WriteOptions::new().with_lz4()));
    let r = PartitionReader::open(f.path()).unwrap();

    // Meaningful only if some columns compressed and some did not.
    let compressed: Vec<bool> = (0..cols.len()).map(|c| r.is_column_compressed(c)).collect();
    assert!(
        compressed.iter().any(|&x| x) && compressed.iter().any(|&x| !x),
        "expected a mix of compressed and verbatim columns, got {compressed:?}"
    );

    let mut buf = Vec::new();
    for (c, (name, vals)) in cols.iter().enumerate() {
        r.decode_column_into(c, &mut buf).unwrap();
        assert_eq!(&buf, vals, "column {c} ({name})");
        for i in [0usize, n / 3, n - 1] {
            assert_eq!(
                r.read_value(c, i).unwrap(),
                vals[i],
                "column {c} ({name}) row {i}"
            );
        }
    }

    // Rows reassemble across the mix.
    for row in [0usize, 1, 255, 256, n - 1] {
        let got = r.read_row(row).unwrap();
        let want: Vec<u64> = cols.iter().map(|(_, v)| v[row]).collect();
        assert_eq!(got, want, "read_row({row})");
    }
}

/// Filtering and materialization work on compressed columns too.
///
/// `materialize_rows` must take the bulk path when any column is compressed —
/// the late path reads cell by cell, and each cell read would decompress the
/// whole column, making a selective query quadratic instead of fast.
#[test]
fn filter_and_materialize_under_compression() {
    let values: Vec<u64> = (0..4000u64).map(|i| 1_000_000 + (i * 13) % 900).collect();
    let mut p = MicroPartition::new(5, 6);
    p.push_column("v", ColumnValues::auto_encode(values.clone()))
        .unwrap();
    let f = write_with(&p, Some(WriteOptions::new().with_lz4()));
    let r = PartitionReader::open(f.path()).unwrap();
    assert!(r.is_column_compressed(0), "test needs a compressed column");

    // Deliberately selective — below LATE_MATERIALIZATION_MAX_SELECTIVITY, so
    // this is exactly the case the compression guard must override.
    let (lo, hi) = (1_000_000, 1_000_050);
    let got = r.filter_offsets(0, lo, hi).unwrap();
    let want: Vec<u32> = values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= lo && **v <= hi)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(got, want, "filter under compression");
    assert!(
        (got.len() as f64 / values.len() as f64) < 0.15,
        "test must be selective enough to exercise the guard"
    );

    let rows = r.materialize_rows(&got).unwrap();
    assert_eq!(rows.len(), want.len());
    for (row, &off) in rows.iter().zip(&want) {
        assert_eq!(row[0], values[off as usize], "materialized row {off}");
    }
}

/// `decode_column_into` agrees with `read_column().to_raw_vec()` everywhere.
///
/// They are separate implementations — the fast path decodes straight from the
/// payload, the other rebuilds a `ColumnValues` first. A divergence means a scan
/// and an inspection disagree about the same bytes.
#[test]
fn scan_path_agrees_with_read_column() {
    for compressed in [false, true] {
        for (name, values) in shapes(1500) {
            let mut p = MicroPartition::new(6, 7);
            p.push_column("v", ColumnValues::auto_encode(values.clone()))
                .unwrap();
            let opts = compressed.then(|| WriteOptions::new().with_lz4());
            let f = write_with(&p, opts);
            let r = PartitionReader::open(f.path()).unwrap();

            let mut fast = Vec::new();
            r.decode_column_into(0, &mut fast).unwrap();
            let slow = r.read_column(0).unwrap().to_raw_vec();
            assert_eq!(fast, slow, "{name} (lz4={compressed}): paths disagree");
            assert_eq!(fast, values, "{name} (lz4={compressed}): wrong values");
        }
    }
}

/// The scan buffer is genuinely reusable: a second decode into the same `Vec`
/// must replace its contents, not append to them.
///
/// `decode_column_into` clears first. Forgetting that is the classic bug with a
/// caller-owned buffer, and it would silently double every scan result.
#[test]
fn decode_buffer_is_reused_not_appended() {
    let a: Vec<u64> = (0..500).collect();
    let b: Vec<u64> = (1000..1300).collect();

    let mut p = MicroPartition::new(7, 8);
    p.push_column("a", ColumnValues::auto_encode(a.clone()))
        .unwrap();
    let fa = write_with(&p, None);
    let ra = PartitionReader::open(fa.path()).unwrap();

    let mut p2 = MicroPartition::new(7, 9);
    p2.push_column("b", ColumnValues::auto_encode(b.clone()))
        .unwrap();
    let fb = write_with(&p2, None);
    let rb = PartitionReader::open(fb.path()).unwrap();

    let mut buf = Vec::new();
    ra.decode_column_into(0, &mut buf).unwrap();
    assert_eq!(buf, a);
    rb.decode_column_into(0, &mut buf).unwrap();
    assert_eq!(buf, b, "second decode appended instead of replacing");
    assert_eq!(buf.len(), b.len());
}

/// A corrupt `uncompressed_len` must not become a huge allocation.
///
/// It is 8 bytes read from the file and handed to the decompressor as an output
/// size. Unguarded, a corrupt value asks for an exabyte-scale `Vec` — an OOM
/// abort triggered by a bad file, which no amount of caller care can prevent.
#[test]
fn corrupt_uncompressed_length_is_capped() {
    let values: Vec<u64> = (0..1000u64).map(|i| i % 8).collect();
    let mut p = MicroPartition::new(8, 10);
    p.push_column("v", ColumnValues::auto_encode(values))
        .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    write_partition_with(&mut buf, &p, WriteOptions::new().with_lz4()).unwrap();

    // Metadata: 32 header + 3 schema ("v"), then
    // encoding[1] bit_width[1] compression[1] pad[5] data_offset[8]
    // data_byte_len[8] uncompressed_len[8]
    let pos = 32 + 3 + 8 + 8 + 8;
    for bad in [u64::MAX, 1u64 << 60, 1u64 << 40] {
        let mut corrupt = buf.clone();
        corrupt[pos..pos + 8].copy_from_slice(&bad.to_le_bytes());
        let len = corrupt.len();
        let crc = crc32fast::hash(&corrupt[..len - 4]);
        corrupt[len - 4..].copy_from_slice(&crc.to_le_bytes());

        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("bad.vvsst");
        std::fs::write(&path, &corrupt).unwrap();
        let r = PartitionReader::open(&path).unwrap();
        assert!(
            r.read_value(0, 0).is_err(),
            "uncompressed_len {bad}: should error, not allocate"
        );
    }
}

/// Reports compression ratios and scan cost. Not an assertion — the numbers in
/// the commit message come from here, so they are reproducible.
#[test]
fn report_compression_and_scan() {
    use std::time::Instant;
    let n = 65536usize;
    println!(
        "\n  {:<18} {:>9} {:>9} {:>9}  {:>10} {:>10}",
        "shape", "raw B", "enc B", "enc+lz4", "scan ns/row", "pt ns"
    );
    for (name, values) in shapes(n) {
        let mut p = MicroPartition::new(9, 11);
        p.push_column("v", ColumnValues::auto_encode(values.clone()))
            .unwrap();
        let plain = write_with(&p, None);
        let packed = write_with(&p, Some(WriteOptions::new().with_lz4()));
        let enc_len = std::fs::metadata(plain.path()).unwrap().len();
        let lz4_len = std::fs::metadata(packed.path()).unwrap().len();

        let r = PartitionReader::open(plain.path()).unwrap();
        let mut buf = Vec::new();
        // Warm the mapping so the first-touch page faults are not counted.
        r.decode_column_into(0, &mut buf).unwrap();
        let mut best = f64::MAX;
        for _ in 0..7 {
            let t = Instant::now();
            r.decode_column_into(0, &mut buf).unwrap();
            best = best.min(t.elapsed().as_nanos() as f64 / n as f64);
        }
        let mut pt = f64::MAX;
        for _ in 0..7 {
            let t = Instant::now();
            for i in (0..n).step_by(97) {
                std::hint::black_box(r.read_value(0, i).unwrap());
            }
            pt = pt.min(t.elapsed().as_nanos() as f64 / n.div_ceil(97) as f64);
        }
        println!(
            "  {:<18} {:>9} {:>9} {:>9}  {:>10.2} {:>10.1}",
            name,
            n * 8,
            enc_len,
            lz4_len,
            best,
            pt
        );
        assert_eq!(buf, values, "{name}");
    }
}
