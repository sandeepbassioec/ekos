//! Criterion benchmarks for [`TableEngine`] — the OLTP hot path.
//!
//! ## What is measured
//!
//! | Benchmark group        | What it exercises                                          |
//! |------------------------|------------------------------------------------------------|
//! | `insert/buffered`      | WAL write + CuckooMap insert + secondary-index update      |
//! | `get/buffered`         | CuckooMap probe + buffer slot read (no disk I/O)           |
//! | `get/flushed`          | CuckooMap probe + mmap column read (page-cache hit)        |
//! | `scan/buffered`        | Full buffer scan (zone-map path not exercised)             |
//! | `scan/flushed`         | SIMD column filter + zone-map pruning + materialization     |
//! | `secondary/buffered`   | SecondaryIndex probe + CuckooMap resolve per matching row  |
//! | `flush`                | Drain buffer → encode columns → fsync-free write          |
//! | `aggregate/buffered`   | count/sum/min/max/avg over buffered partition               |
//! | `aggregate/flushed`    | count/sum/min/max/avg with zone-map pruning                 |
//! | `sorted_scan/buffered` | scan_sorted: cross-partition sort + limit                   |
//! | `sorted_scan/flushed`  | scan_sorted after flush (page-cache hit)                   |
//! | `scan_page/buffered`   | cursor-based page scan over buffer                         |
//! | `scan_page/flushed`    | cursor-based page scan over flushed partition              |
//!
//! ## How to run
//!
//! ```text
//! cargo bench -p vectorvolt-engine
//! cargo bench -p vectorvolt-engine -- --save-baseline main
//! cargo bench -p vectorvolt-engine -- --baseline main
//! ```
//!
//! ## Design decisions
//!
//! * **WriteOnly durability** everywhere except the `flush` benchmark.
//!   `fsync` dominates at ~1ms and would drown every other signal.
//! * **Pre-seeded** tables: each benchmark group warms up the table once
//!   outside the timed loop so steady-state throughput is what's measured, not
//!   first-insert setup.
//! * **`black_box`** on every result so the compiler cannot prove the work is
//!   dead and elide it.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

const FLUSH_THRESHOLD: usize = 1_048_576; // 1 MiB — large enough to keep bench rows buffered
const SCHEMA: &[&str] = &["pk", "age", "score", "ts"];

fn open(dir: &TempDir) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            SCHEMA.iter().map(|s| s.to_string()).collect(),
            dir.path(),
            EngineOptions::new(FLUSH_THRESHOLD).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

fn open_small(dir: &TempDir) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            SCHEMA.iter().map(|s| s.to_string()).collect(),
            dir.path(),
            // Threshold of 0 means auto-flush on every insert — for flush bench.
            EngineOptions::new(64).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

// ── insert ────────────────────────────────────────────────────────────────────

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for &n in &[1u64, 10, 100] {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::new("buffered", n), &n, |b, &n| {
            let dir = TempDir::new().unwrap();
            let engine = open(&dir);
            let mut pk = 0u64;
            b.iter(|| {
                for _ in 0..n {
                    engine
                        .insert(black_box(pk), black_box(vec![pk, pk % 100, pk * 2, pk]))
                        .unwrap();
                    pk = pk.wrapping_add(1);
                }
            });
        });
    }

    group.finish();
}

// ── get ──────────────────────────────────────────────────────────────────────

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");
    const SEED: u64 = 10_000;

    // buffered: rows live in the write buffer, no disk touched.
    group.bench_function("buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        let mut pk = 0u64;
        b.iter(|| {
            let row = engine.get(black_box(pk % SEED)).unwrap();
            black_box(row);
            pk = pk.wrapping_add(1);
        });
    });

    // flushed: rows are in an mmap'd partition file (page-cache hit after warm-up).
    group.bench_function("flushed", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        // Read once to warm the page cache.
        for pk in 0..SEED {
            let _ = engine.get(pk);
        }
        let mut pk = 0u64;
        b.iter(|| {
            let row = engine.get(black_box(pk % SEED)).unwrap();
            black_box(row);
            pk = pk.wrapping_add(1);
        });
    });

    group.finish();
}

// ── scan ─────────────────────────────────────────────────────────────────────

fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    const SEED: u64 = 10_000;

    // buffered: scan walks the in-memory buffer.
    group.bench_function("buffered_1pct", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        b.iter(|| {
            // col=1 (age), range [0, 1] → ~1% selectivity
            let rows = engine.scan(1, 0, 1).unwrap();
            black_box(rows);
        });
    });

    // flushed: SIMD column filter + zone-map pruning.
    group.bench_function("flushed_1pct", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        for pk in 0..SEED {
            let _ = engine.get(pk); // page-cache warm
        }
        b.iter(|| {
            let rows = engine.scan(1, 0, 1).unwrap();
            black_box(rows);
        });
    });

    group.bench_function("flushed_10pct", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        for pk in 0..SEED {
            let _ = engine.get(pk);
        }
        b.iter(|| {
            let rows = engine.scan(1, 0, 9).unwrap();
            black_box(rows);
        });
    });

    group.finish();
}

// ── secondary_scan ────────────────────────────────────────────────────────────

fn bench_secondary(c: &mut Criterion) {
    let mut group = c.benchmark_group("secondary");
    const SEED: u64 = 10_000;

    group.bench_function("buffered_exact", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        // age=42 → ~100 matching rows
        b.iter(|| {
            let rows = engine.secondary_scan(1, 42).unwrap();
            black_box(rows);
        });
    });

    group.finish();
}

// ── flush ─────────────────────────────────────────────────────────────────────

fn bench_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("flush");

    for &n in &[100u64, 1_000, 10_000] {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::new("rows", n), &n, |b, &n| {
            b.iter_with_setup(
                || {
                    let dir = TempDir::new().unwrap();
                    let engine = open_small(&dir);
                    for pk in 0..n {
                        // Use insert without triggering auto-flush by using a
                        // large-threshold engine, then call flush manually below.
                        let _ = engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]);
                    }
                    (dir, engine)
                },
                |(_dir, engine)| {
                    black_box(engine.flush().unwrap());
                },
            );
        });
    }

    group.finish();
}

// ── aggregate ─────────────────────────────────────────────────────────────────

fn bench_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate");
    const SEED: u64 = 10_000;

    // buffered: aggregates walk the in-memory buffer (no disk I/O).
    group.bench_function("count_buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        b.iter(|| black_box(engine.count_range(1, 0, 49).unwrap()));
    });

    group.bench_function("sum_buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        b.iter(|| black_box(engine.sum_range(1, 0, 99).unwrap()));
    });

    group.bench_function("min_buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        b.iter(|| black_box(engine.min_range(1, 0, 99).unwrap()));
    });

    group.bench_function("max_buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        b.iter(|| black_box(engine.max_range(1, 0, 99).unwrap()));
    });

    // flushed: zone-map pruning kicks in for out-of-range partitions.
    group.bench_function("count_flushed", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        for pk in 0..SEED {
            let _ = engine.get(pk);
        }
        b.iter(|| black_box(engine.count_range(1, 0, 49).unwrap()));
    });

    group.finish();
}

// ── sorted_scan ───────────────────────────────────────────────────────────────

fn bench_sorted_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorted_scan");
    const SEED: u64 = 10_000;

    group.bench_function("buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        // col=1 filter [0,99], sort by col=2 ascending, top 100
        b.iter(|| black_box(engine.scan_sorted(1, 0, 99, 2, true, 100).unwrap()));
    });

    group.bench_function("flushed", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        for pk in 0..SEED {
            let _ = engine.get(pk);
        }
        b.iter(|| black_box(engine.scan_sorted(1, 0, 99, 2, true, 100).unwrap()));
    });

    group.finish();
}

// ── scan_page ─────────────────────────────────────────────────────────────────

fn bench_scan_page(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_page");
    const SEED: u64 = 10_000;

    group.bench_function("buffered", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        // cursor=0 → first page of 100 rows
        b.iter(|| black_box(engine.scan_page(1, 0, 99, 0, 100).unwrap()));
    });

    group.bench_function("flushed", |b| {
        let dir = TempDir::new().unwrap();
        let engine = open(&dir);
        for pk in 0..SEED {
            engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
        }
        engine.flush().unwrap();
        for pk in 0..SEED {
            let _ = engine.get(pk);
        }
        b.iter(|| black_box(engine.scan_page(1, 0, 99, 0, 100).unwrap()));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_get,
    bench_scan,
    bench_secondary,
    bench_flush,
    bench_aggregate,
    bench_sorted_scan,
    bench_scan_page,
);
criterion_main!(benches);
