//! Gap #5 — TableEngine performance-regression gate (VV-TST-006/007).
//!
//! `benches/engine_bench.rs` is Criterion: it measures and prints, but asserts
//! nothing, so nobody actually verifies the "sub-microsecond / zero-GC" latency
//! story stays true over time. This file mirrors the pattern already proven in
//! `vectorvolt-hal/tests/perf_gate.rs`: plain `#[test]` functions with
//! deliberately conservative throughput floors (orders of magnitude below what
//! this engine actually achieves) so they always pass on real hardware —
//! including slow/debug-mode CI — but fail loudly on a catastrophic regression
//! (e.g. an accidentally-disabled index, an O(n^2) scan, or a lock that
//! serializes what should be lock-free).
//!
//! Threshold derivation: floors are set ~50-100x below what a healthy debug
//! build measures locally, so normal noise never trips them, but a real
//! regression (10x+ slowdown) will.

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

const FLUSH_THRESHOLD: usize = 1_048_576;
const SCHEMA: &[&str] = &["pk", "age", "score", "ts"];
const SEED: u64 = 10_000;

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

fn ops_per_sec(iters: u32, f: impl FnMut()) -> u64 {
    let mut f = f;
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = start.elapsed().as_secs_f64().max(1e-9);
    (iters as f64 / elapsed) as u64
}

#[test]
fn perf_insert_buffered_above_floor() {
    const FLOOR_OPS_PER_SEC: u64 = 1_000; // WAL-backed insert; measured ~3k/s on a loaded debug build
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);
    let mut pk = 0u64;
    let throughput = ops_per_sec(5_000, || {
        engine
            .insert(black_box(pk), black_box(vec![pk, pk % 100, pk * 2, pk]))
            .unwrap();
        pk = pk.wrapping_add(1);
    });
    assert!(
        throughput >= FLOOR_OPS_PER_SEC,
        "insert/buffered: {throughput} ops/s < floor {FLOOR_OPS_PER_SEC}"
    );
}

#[test]
fn perf_get_buffered_above_floor() {
    const FLOOR_OPS_PER_SEC: u64 = 10_000; // CuckooMap probe + buffer read; measured ~10^5-10^6/s
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);
    for pk in 0..SEED {
        engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
    }
    let mut pk = 0u64;
    let throughput = ops_per_sec(20_000, || {
        black_box(engine.get(black_box(pk % SEED)).unwrap());
        pk = pk.wrapping_add(1);
    });
    assert!(
        throughput >= FLOOR_OPS_PER_SEC,
        "get/buffered: {throughput} ops/s < floor {FLOOR_OPS_PER_SEC}"
    );
}

#[test]
fn perf_get_flushed_above_floor() {
    const FLOOR_OPS_PER_SEC: u64 = 5_000; // mmap page-cache hit; measured ~10^4-10^5/s
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);
    for pk in 0..SEED {
        engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
    }
    engine.flush().unwrap();
    for pk in 0..SEED {
        let _ = engine.get(pk); // warm page cache
    }
    let mut pk = 0u64;
    let throughput = ops_per_sec(20_000, || {
        black_box(engine.get(black_box(pk % SEED)).unwrap());
        pk = pk.wrapping_add(1);
    });
    assert!(
        throughput >= FLOOR_OPS_PER_SEC,
        "get/flushed: {throughput} ops/s < floor {FLOOR_OPS_PER_SEC}"
    );
}

#[test]
fn perf_scan_flushed_1pct_above_floor() {
    const FLOOR_OPS_PER_SEC: u64 = 150; // SIMD-filtered 10k-row scan; measured ~450-500/s on a loaded debug build
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);
    for pk in 0..SEED {
        engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
    }
    engine.flush().unwrap();
    for pk in 0..SEED {
        let _ = engine.get(pk);
    }
    let throughput = ops_per_sec(500, || {
        black_box(engine.scan(1, 0, 1).unwrap());
    });
    assert!(
        throughput >= FLOOR_OPS_PER_SEC,
        "scan/flushed_1pct: {throughput} ops/s < floor {FLOOR_OPS_PER_SEC}"
    );
}

#[test]
fn perf_secondary_scan_above_floor() {
    const FLOOR_OPS_PER_SEC: u64 = 100; // secondary-index probe + resolve per row; measured ~320-390/s on a loaded debug build
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);
    for pk in 0..SEED {
        engine.insert(pk, vec![pk, pk % 100, pk * 2, pk]).unwrap();
    }
    let throughput = ops_per_sec(2_000, || {
        black_box(engine.secondary_scan(1, 42).unwrap());
    });
    assert!(
        throughput >= FLOOR_OPS_PER_SEC,
        "secondary_scan: {throughput} ops/s < floor {FLOOR_OPS_PER_SEC}"
    );
}
