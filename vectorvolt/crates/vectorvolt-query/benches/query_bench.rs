//! Criterion benchmarks for the VectorVolt query DSL.
//!
//! Measures the full pipeline: tokenize → parse → execute against a live engine.
//!
//! ## Benchmark groups
//!
//! | Group          | What it exercises                                       |
//! |----------------|---------------------------------------------------------|
//! | `dsl/read`     | GET, SCAN, COUNT, AVG, SCAN SORTED, SCAN PAGE           |
//! | `dsl/write`    | INSERT, UPSERT, UPDATE FIELD, DELETE RANGE              |
//! | `dsl/parse`    | parse-only (no engine I/O) — isolates tokenizer cost    |

use std::hint::black_box;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_query::{parse, run};

const SCHEMA: &[&str] = &["pk", "val", "score"];
const SEED: u64 = 5_000;

fn open_seeded() -> (Arc<TableEngine>, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            SCHEMA.iter().map(|s| s.to_string()).collect(),
            dir.path(),
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    for pk in 0..SEED {
        engine.insert(pk, vec![pk, pk % 100, pk * 2]).unwrap();
    }
    (engine, dir)
}

fn open_empty() -> (Arc<TableEngine>, TempDir) {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            SCHEMA.iter().map(|s| s.to_string()).collect(),
            dir.path(),
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    (engine, dir)
}

// ── read queries ──────────────────────────────────────────────────────────────

fn bench_dsl_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/read");
    let (engine, _dir) = open_seeded();

    group.bench_function("get", |b| {
        let mut pk = 0u64;
        b.iter(|| {
            let q = format!("GET {}", pk % SEED);
            black_box(run(&engine, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    group.bench_function("scan", |b| {
        b.iter(|| black_box(run(&engine, "SCAN col=1 lo=0 hi=10").unwrap()));
    });

    group.bench_function("count", |b| {
        b.iter(|| black_box(run(&engine, "COUNT col=1 lo=0 hi=99").unwrap()));
    });

    group.bench_function("avg", |b| {
        b.iter(|| black_box(run(&engine, "AVG col=1 lo=0 hi=99").unwrap()));
    });

    group.bench_function("scan_sorted", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "SCAN SORTED col=1 lo=0 hi=99 sort=2 asc=1 limit=50",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("scan_page", |b| {
        b.iter(|| black_box(run(&engine, "SCAN PAGE col=1 lo=0 hi=99 after=0 limit=50").unwrap()));
    });

    group.finish();
}

// ── write queries ─────────────────────────────────────────────────────────────

fn bench_dsl_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/write");
    let (engine, _dir) = open_empty();

    group.bench_function("insert", |b| {
        let mut pk = 100_000u64;
        b.iter(|| {
            let q = format!("INSERT {} {} {} {}", pk, pk, pk % 100, pk * 2);
            black_box(run(&engine, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    group.bench_function("upsert", |b| {
        let mut pk = 200_000u64;
        b.iter(|| {
            let q = format!("UPSERT {} {} {} {}", pk, pk, pk % 100, pk * 2);
            black_box(run(&engine, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    group.bench_function("update_field", |b| {
        run(&engine, "INSERT 1 1 10 20").ok();
        b.iter(|| black_box(run(&engine, "UPDATE FIELD 1 col=1 val=42").unwrap()));
    });

    group.bench_function("delete_range", |b| {
        b.iter_with_setup(
            || {
                for pk in 300_000u64..300_010 {
                    run(&engine, &format!("INSERT {} {} 5 0", pk, pk)).ok();
                }
            },
            |_| black_box(run(&engine, "DELETE RANGE col=0 lo=300000 hi=300009").unwrap()),
        );
    });

    group.finish();
}

// ── parse-only ────────────────────────────────────────────────────────────────

fn bench_dsl_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/parse");

    let stmts: &[(&str, &str)] = &[
        ("get", "GET 42"),
        ("scan", "SCAN col=1 lo=0 hi=100"),
        ("count", "COUNT col=1 lo=0 hi=100"),
        (
            "scan_sorted",
            "SCAN SORTED col=1 lo=0 hi=99 sort=2 asc=1 limit=50",
        ),
        ("upsert", "UPSERT 1 1 10 20"),
        ("delete_range", "DELETE RANGE col=0 lo=1 hi=100"),
    ];

    for &(name, stmt) in stmts {
        group.bench_function(name, |b| {
            b.iter(|| black_box(parse(stmt).unwrap()));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_dsl_read, bench_dsl_write, bench_dsl_parse);
criterion_main!(benches);
