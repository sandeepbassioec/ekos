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

// ── analytics queries (Layers 50-53) ─────────────────────────────────────────

fn bench_dsl_analytics(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/analytics");
    let (engine, _dir) = open_seeded();

    group.bench_function("histogram", |b| {
        b.iter(|| black_box(run(&engine, "HISTOGRAM col=1 lo=0 hi=99 buckets=10").unwrap()));
    });

    group.bench_function("percentile_p50", |b| {
        b.iter(|| black_box(run(&engine, "PERCENTILE col=1 lo=0 hi=99 p=50").unwrap()));
    });

    group.bench_function("percentile_p99", |b| {
        b.iter(|| black_box(run(&engine, "PERCENTILE col=1 lo=0 hi=99 p=99").unwrap()));
    });

    group.bench_function("group_count", |b| {
        b.iter(|| {
            black_box(run(&engine, "GROUP COUNT filter_col=1 lo=0 hi=99 group_col=1").unwrap())
        });
    });

    group.bench_function("group_sum", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "GROUP SUM filter_col=1 lo=0 hi=99 group_col=1 agg_col=2",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("group_min", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "GROUP MIN filter_col=1 lo=0 hi=99 group_col=1 agg_col=2",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("group_max", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "GROUP MAX filter_col=1 lo=0 hi=99 group_col=1 agg_col=2",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("window_running_sum", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "WINDOW RUNNING_SUM filter_col=1 lo=0 hi=9 agg_col=2",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("window_running_count", |b| {
        b.iter(|| black_box(run(&engine, "WINDOW RUNNING_COUNT filter_col=1 lo=0 hi=9").unwrap()));
    });

    group.bench_function("window_rank", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "WINDOW RANK filter_col=1 lo=0 hi=9 sort_col=2 asc=1",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("window_dense_rank", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "WINDOW DENSE_RANK filter_col=1 lo=0 hi=9 sort_col=2 asc=1",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("distinct", |b| {
        b.iter(|| black_box(run(&engine, "DISTINCT col=1 lo=0 hi=99").unwrap()));
    });

    group.bench_function("value_counts", |b| {
        b.iter(|| black_box(run(&engine, "VALUE_COUNTS col=1 lo=0 hi=99").unwrap()));
    });

    group.finish();
}

// ── predicate mutate + scan queries (Layers 54-57) ────────────────────────────

fn bench_dsl_predicate(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/predicate");
    let (engine, _dir) = open_seeded();

    group.bench_function("update_where", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "UPDATE WHERE filter_col=1 lo=0 hi=9 target_col=2 val=42",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("increment_where", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "INCREMENT WHERE filter_col=1 lo=0 hi=9 target_col=2 delta=1",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("decrement_where", |b| {
        b.iter(|| {
            black_box(
                run(
                    &engine,
                    "DECREMENT WHERE filter_col=1 lo=0 hi=9 target_col=2 delta=1",
                )
                .unwrap(),
            )
        });
    });

    group.bench_function("scan_where_1pred", |b| {
        b.iter(|| black_box(run(&engine, "SCAN WHERE col=1 lo=0 hi=9").unwrap()));
    });

    group.bench_function("scan_where_2pred", |b| {
        b.iter(|| {
            black_box(run(&engine, "SCAN WHERE col=1 lo=0 hi=49 AND col=2 lo=0 hi=100").unwrap())
        });
    });

    group.bench_function("scan_project", |b| {
        b.iter(|| black_box(run(&engine, "SCAN PROJECT col=1 lo=0 hi=99 cols=0 2").unwrap()));
    });

    group.finish();
}

// ── conditional writes + batch + TTL (Layers 58-60) ──────────────────────────

fn bench_dsl_cond_batch_ttl(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/cond_batch_ttl");
    let (engine, _dir) = open_seeded();

    // insert_if_absent on existing pk → fast false return
    group.bench_function("insert_if_absent_existing", |b| {
        b.iter(|| black_box(run(&engine, "INSERT IF_ABSENT 1 1 1 2").unwrap()));
    });

    // insert_if_absent on a fresh pk each time
    group.bench_function("insert_if_absent_new", |b| {
        let (e2, _d2) = open_empty();
        let mut pk = 500_000u64;
        b.iter(|| {
            let q = format!("INSERT IF_ABSENT {} {} {} {}", pk, pk, pk % 100, pk * 2);
            black_box(run(&e2, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    // CAS with wrong expected → fast mismatch return
    group.bench_function("cas_mismatch", |b| {
        b.iter(|| {
            // pk=1 has values [1,1,2]; expected [9,9,9] won't match
            black_box(run(&engine, "CAS 1 expected 9 9 9 new 1 99 200").unwrap())
        });
    });

    group.bench_function("batch_insert_10", |b| {
        b.iter_with_setup(
            || {
                // build a fresh batch statement each setup
                let rows: String = (0u64..10)
                    .map(|i| format!("row {} {} {} {}", 900_000 + i, 900_000 + i, i % 100, i * 2))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("BATCH INSERT {rows}")
            },
            |q| black_box(run(&engine, &q).unwrap()),
        );
    });

    // TTL: insert + get + scan + purge (nothing to purge)
    group.bench_function("insert_ttl", |b| {
        let (e3, _d3) = open_empty();
        let mut pk = 700_000u64;
        b.iter(|| {
            let q = format!(
                "INSERT TTL {} {} {} {} expire={}",
                pk,
                pk,
                pk % 100,
                pk * 2,
                u64::MAX
            );
            black_box(run(&e3, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    group.bench_function("get_ttl", |b| {
        let (e4, _d4) = open_empty();
        for pk in 0u64..1000 {
            run(
                &e4,
                &format!(
                    "INSERT TTL {} {} {} {} expire={}",
                    pk,
                    pk,
                    pk % 100,
                    pk * 2,
                    u64::MAX
                ),
            )
            .ok();
        }
        let mut pk = 0u64;
        b.iter(|| {
            let q = format!("GET TTL {} now=0", pk % 1000);
            black_box(run(&e4, &q).unwrap());
            pk = pk.wrapping_add(1);
        });
    });

    group.bench_function("scan_ttl", |b| {
        let (e5, _d5) = open_empty();
        for pk in 0u64..1000 {
            run(
                &e5,
                &format!(
                    "INSERT TTL {} {} {} {} expire={}",
                    pk,
                    pk,
                    pk % 100,
                    pk * 2,
                    u64::MAX
                ),
            )
            .ok();
        }
        b.iter(|| black_box(run(&e5, "SCAN TTL now=0").unwrap()));
    });

    // purge with nothing expired
    group.bench_function("purge_expired_none", |b| {
        let (e6, _d6) = open_empty();
        for pk in 0u64..1000 {
            run(
                &e6,
                &format!(
                    "INSERT TTL {} {} {} {} expire={}",
                    pk,
                    pk,
                    pk % 100,
                    pk * 2,
                    u64::MAX
                ),
            )
            .ok();
        }
        b.iter(|| black_box(run(&e6, "PURGE EXPIRED now=0").unwrap()));
    });

    group.finish();
}

// ── parse-only for new statements ─────────────────────────────────────────────

fn bench_dsl_parse_layer50(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl/parse_layer50");

    let stmts: &[(&str, &str)] = &[
        ("histogram", "HISTOGRAM col=1 lo=0 hi=99 buckets=10"),
        ("percentile", "PERCENTILE col=1 lo=0 hi=99 p=50"),
        (
            "group_count",
            "GROUP COUNT filter_col=1 lo=0 hi=99 group_col=1",
        ),
        (
            "group_sum",
            "GROUP SUM filter_col=1 lo=0 hi=99 group_col=1 agg_col=2",
        ),
        (
            "window_running_sum",
            "WINDOW RUNNING_SUM filter_col=1 lo=0 hi=9 agg_col=2",
        ),
        (
            "window_rank",
            "WINDOW RANK filter_col=1 lo=0 hi=9 sort_col=2 asc=1",
        ),
        ("distinct", "DISTINCT col=1 lo=0 hi=99"),
        ("value_counts", "VALUE_COUNTS col=1 lo=0 hi=99"),
        (
            "update_where",
            "UPDATE WHERE filter_col=1 lo=0 hi=9 target_col=2 val=42",
        ),
        (
            "increment_where",
            "INCREMENT WHERE filter_col=1 lo=0 hi=9 target_col=2 delta=1",
        ),
        ("scan_where", "SCAN WHERE col=1 lo=0 hi=9"),
        ("scan_project", "SCAN PROJECT col=1 lo=0 hi=99 cols=0 2"),
        ("insert_if_absent", "INSERT IF_ABSENT 42 42 50 100"),
        ("cas", "CAS 1 expected 9 9 9 new 1 99 200"),
        ("batch_insert", "BATCH INSERT row 1 1 50 100 row 2 2 60 120"),
        ("insert_ttl", "INSERT TTL 1 1 50 100 expire=9999999"),
        ("get_ttl", "GET TTL 1 now=0"),
        ("scan_ttl", "SCAN TTL now=0"),
        ("purge_expired", "PURGE EXPIRED now=500"),
    ];

    for &(name, stmt) in stmts {
        group.bench_function(name, |b| {
            b.iter(|| black_box(parse(stmt).unwrap()));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dsl_read,
    bench_dsl_write,
    bench_dsl_parse,
    bench_dsl_analytics,
    bench_dsl_predicate,
    bench_dsl_cond_batch_ttl,
    bench_dsl_parse_layer50,
);
criterion_main!(benches);
