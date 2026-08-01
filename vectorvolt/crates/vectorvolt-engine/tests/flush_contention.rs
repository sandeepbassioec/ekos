//! How much does holding the write-buffer lock across a flush cost?
//!
//! The previous fix holds that lock from drain through the index rewrite, which
//! includes writing the `.vvsst` file. That closed a correctness hole and it
//! serialises every insert against file I/O. This measures the cost rather than
//! assuming it is either negligible or fatal.

use std::sync::Arc;
use std::time::Instant;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn engine(dir: &TempDir, threshold: usize) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "a".into(), "b".into()],
            dir.path(),
            EngineOptions::new(threshold).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

fn run(threads: u64, per_thread: u64, threshold: usize) -> f64 {
    let dir = TempDir::new().unwrap();
    let m = engine(&dir, threshold);
    let t0 = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let m = Arc::clone(&m);
            std::thread::spawn(move || {
                for i in 0..per_thread {
                    let pk = i * threads + t;
                    m.insert(pk, vec![pk, t, i]).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let total = (threads * per_thread) as f64;
    t0.elapsed().as_nanos() as f64 / total
}

/// Run manually: `cargo test --release -p vectorvolt-engine --test flush_contention
/// -- --ignored --nocapture`
///
/// `#[ignore]` because it is a measurement, not an assertion — it prints numbers
/// and asserts nothing, so it does not belong in the gate. At 20 000 inserts x 8
/// threads in a debug build it is also slow enough to distort the very contention
/// it measures when the whole workspace is running.
///
/// A previous version of this comment noted a `CuckooMap::insert` failure at
/// ~20 000 keys. That was fixed: the load factor was cut to 43.75% (below the
/// 50% two-way cuckoo threshold), both hash functions gained xor-shift folding to
/// handle power-of-two-strided keys, `homes()` now guarantees distinct buckets,
/// and the displacement loop was fixed to always advance to the OTHER home instead
/// of cycling. See the load-factor and hash-quality tests in `cuckoo_map.rs` for
/// the regression coverage.
#[ignore = "manual measurement only — prints numbers, asserts nothing"]
#[test]
fn report_insert_throughput_under_flush_pressure() {
    println!("\n  {:<28} {:>12}", "configuration", "ns/insert");
    // No flushes at all: pure buffered insert, the lock's baseline cost.
    println!(
        "  {:<28} {:>12.0}",
        "1 thread, no flush",
        run(1, 20_000, 100_000_000)
    );
    println!(
        "  {:<28} {:>12.0}",
        "8 threads, no flush",
        run(8, 2_500, 100_000_000)
    );
    // Flushes every 256 rows: the lock is now held across real file I/O.
    println!(
        "  {:<28} {:>12.0}",
        "1 thread, flush/256",
        run(1, 20_000, 256)
    );
    println!(
        "  {:<28} {:>12.0}",
        "8 threads, flush/256",
        run(8, 2_500, 256)
    );
    println!(
        "  {:<28} {:>12.0}",
        "8 threads, flush/64",
        run(8, 2_500, 64)
    );
}
