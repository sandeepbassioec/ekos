//! Criterion benchmarks for the read paths (VV-TST-007).
//!
//! ## Why this exists, given there was already a timing test
//!
//! `report_compression_and_scan` prints numbers, and those numbers were wrong for
//! two commits without anyone noticing: a per-row `String` allocation made frame
//! decode look 15x slower than it is, and the figure was quoted in a commit
//! message as a real cost of the encodings.
//!
//! A `best-of-7` loop around `Instant::now()` cannot catch that. It reports a
//! number with no variance, no outlier detection, and no comparison against the
//! last run — so a regression is only visible to someone who happens to remember
//! the previous value. Criterion keeps a baseline and tells you when a change
//! moved something.
//!
//! Run with:
//! ```text
//! cargo bench -p vectorvolt-storage
//! ```
//!
//! ## What is measured, and what is deliberately not
//!
//! **Measured:** decoding a column that is already in the page cache, and point
//! reads against it. That isolates the CPU cost of an encoding, which is the
//! thing an encoding choice actually controls.
//!
//! **Not measured:** cold-cache reads, disk I/O, or first-touch page faults. Those
//! are dominated by the storage device and would drown the signal — a benchmark
//! that mostly measures an NVMe drive tells you nothing about whether bit-packing
//! is worth it. The partition is opened and fully read once before timing starts,
//! for exactly that reason.
//!
//! This means the numbers are an upper bound on how good the encodings look. On a
//! cold read the decode cost is a smaller fraction of the total, so the encodings
//! look *better* there, not worse.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::TempDir;
use vectorvolt_storage::{
    ColumnValues, MicroPartition, PartitionReader, WriteOptions, write_partition_with,
};

const ROWS: usize = 65_536;

/// Value shapes, each chosen so a different encoding wins.
fn shapes() -> Vec<(&'static str, Vec<u64>)> {
    let distinct = [0u64, u64::MAX / 4, u64::MAX / 2, u64::MAX];
    vec![
        // BlockedFrame — sorted keys, the primary-key shape.
        ("sequential", (0..ROWS as u64).collect()),
        // Frame — narrow range around a large base, the timestamp shape.
        (
            "narrow",
            (0..ROWS as u64)
                .map(|i| 10_000_000_000 + i % 1000)
                .collect(),
        ),
        // Rle — long runs.
        (
            "runs",
            (0..ROWS as u64).map(|i| (i / 1024) * 9_999_991).collect(),
        ),
        // Dict — few values, far apart.
        (
            "wide_low_card",
            (0..ROWS).map(|i| distinct[(i * 7) % 4]).collect(),
        ),
        // Raw — the baseline every other encoding is compared against.
        (
            "incompressible",
            (0..ROWS as u64)
                .map(|i| {
                    let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    x ^= x >> 29;
                    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                    x ^ (x >> 32)
                })
                .collect(),
        ),
    ]
}

fn write(dir: &TempDir, name: &str, values: &[u64], lz4: bool) -> PartitionReader {
    let mut p = MicroPartition::new(0, 1);
    p.push_column(name, ColumnValues::auto_encode(values.to_vec()))
        .unwrap();
    let path = dir
        .path()
        .join(format!("{name}{}.vvsst", if lz4 { "_lz4" } else { "" }));
    let mut f = std::fs::File::create(&path).unwrap();
    let opts = if lz4 {
        WriteOptions::new().with_lz4()
    } else {
        WriteOptions::default()
    };
    write_partition_with(&mut f, &p, opts).unwrap();
    drop(f);

    let r = PartitionReader::open(&path).unwrap();
    // Warm the mapping. Measuring first-touch page faults would measure the OS.
    let mut warm = Vec::new();
    r.decode_column_into(0, &mut warm).unwrap();
    r
}

/// Full-column decode: the scan path.
fn bench_scan(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let mut g = c.benchmark_group("scan_decode");
    g.throughput(Throughput::Elements(ROWS as u64));

    for (name, values) in shapes() {
        let r = write(&dir, name, &values, false);
        let mut buf = Vec::with_capacity(ROWS);
        g.bench_with_input(BenchmarkId::from_parameter(name), &r, |b, r| {
            b.iter(|| {
                r.decode_column_into(0, &mut buf).unwrap();
                black_box(buf.len())
            })
        });
    }
    g.finish();
}

/// Point reads: the O(1) path VectorVolt's read architecture rests on.
///
/// Strided rather than sequential, so the hardware prefetcher cannot turn a
/// random-access benchmark into a sequential one and make every encoding look
/// identical.
fn bench_point_read(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let mut g = c.benchmark_group("point_read");

    const STRIDE: usize = 4_099; // prime, so it walks the whole column
    let probes: Vec<usize> = (0..1024).map(|i| (i * STRIDE) % ROWS).collect();
    g.throughput(Throughput::Elements(probes.len() as u64));

    for (name, values) in shapes() {
        let r = write(&dir, name, &values, false);
        g.bench_with_input(BenchmarkId::from_parameter(name), &r, |b, r| {
            b.iter(|| {
                let mut acc = 0u64;
                for &row in &probes {
                    acc = acc.wrapping_add(r.read_value(0, row).unwrap());
                }
                black_box(acc)
            })
        });
    }
    g.finish();
}

/// What LZ4 costs on the read side.
///
/// Compression is opt-in precisely because it converts an O(1) point read into an
/// O(column) one. This puts a number on both halves of that trade rather than
/// leaving it as an assertion in a doc comment.
fn bench_compression_cost(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let mut g = c.benchmark_group("lz4_scan_cost");
    g.throughput(Throughput::Elements(ROWS as u64));

    for (name, values) in shapes() {
        for lz4 in [false, true] {
            let r = write(&dir, name, &values, lz4);
            let mut buf = Vec::with_capacity(ROWS);
            let id = format!("{name}/{}", if lz4 { "lz4" } else { "plain" });
            g.bench_with_input(BenchmarkId::from_parameter(id), &r, |b, r| {
                b.iter(|| {
                    r.decode_column_into(0, &mut buf).unwrap();
                    black_box(buf.len())
                })
            });
        }
    }
    g.finish();
}

/// Encoding selection itself — paid on every flush, so it is on the write path.
fn bench_encode(c: &mut Criterion) {
    let mut g = c.benchmark_group("auto_encode");
    g.throughput(Throughput::Elements(ROWS as u64));
    for (name, values) in shapes() {
        g.bench_with_input(BenchmarkId::from_parameter(name), &values, |b, values| {
            b.iter(|| black_box(ColumnValues::auto_encode(values.clone())))
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_scan,
    bench_point_read,
    bench_compression_cost,
    bench_encode
);
criterion_main!(benches);
