//! Layer 171 — vvctl bench scan/insert/latency
//! Tests the bench command with various configurations.

use vvctl::commands::bench::{BenchArgs, run};

// 1. bench defaults (10 rows, no_scan=false) runs Ok
#[test]
fn bench_defaults_ok() {
    let args = BenchArgs {
        rows: 100,
        warmup: 10,
        flush_every: 0,
        no_get: false,
        no_scan: false,
        no_fsync: false,
        buffered: true, // use buffered to be fast in tests
    };
    assert!(run(args).is_ok());
}

// 2. bench with no_scan=true runs Ok
#[test]
fn bench_no_scan_ok() {
    let args = BenchArgs {
        rows: 100,
        warmup: 10,
        flush_every: 0,
        no_get: false,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 3. bench with no_get=true runs Ok
#[test]
fn bench_no_get_ok() {
    let args = BenchArgs {
        rows: 100,
        warmup: 10,
        flush_every: 0,
        no_get: true,
        no_scan: false,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 4. bench with no_get and no_scan
#[test]
fn bench_no_get_no_scan_ok() {
    let args = BenchArgs {
        rows: 50,
        warmup: 5,
        flush_every: 0,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 5. bench with flush_every=10 runs Ok
#[test]
fn bench_flush_every_10_ok() {
    let args = BenchArgs {
        rows: 50,
        warmup: 5,
        flush_every: 10,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 6. bench with rows=0 returns Err
#[test]
fn bench_rows_zero_returns_err() {
    let args = BenchArgs {
        rows: 0,
        warmup: 0,
        flush_every: 0,
        no_get: false,
        no_scan: false,
        no_fsync: false,
        buffered: false,
    };
    assert!(run(args).is_err());
}

// 7. bench with warmup >= rows returns Err
#[test]
fn bench_warmup_ge_rows_returns_err() {
    let args = BenchArgs {
        rows: 10,
        warmup: 10,
        flush_every: 0,
        no_get: false,
        no_scan: false,
        no_fsync: false,
        buffered: false,
    };
    assert!(run(args).is_err());
}

// 8. bench small rows completes quickly
#[test]
fn bench_small_rows_ok() {
    let args = BenchArgs {
        rows: 20,
        warmup: 2,
        flush_every: 0,
        no_get: false,
        no_scan: false,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 9. bench no_fsync=true runs Ok
#[test]
fn bench_no_fsync_ok() {
    let args = BenchArgs {
        rows: 50,
        warmup: 5,
        flush_every: 0,
        no_get: true,
        no_scan: true,
        no_fsync: true,
        buffered: false,
    };
    assert!(run(args).is_ok());
}

// 10. bench buffered=true runs Ok
#[test]
fn bench_buffered_ok() {
    let args = BenchArgs {
        rows: 50,
        warmup: 5,
        flush_every: 0,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 11. bench with flush_every=1 runs Ok
#[test]
fn bench_flush_every_1_ok() {
    let args = BenchArgs {
        rows: 20,
        warmup: 2,
        flush_every: 1,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}

// 12. bench warmup=0 with rows > 0 runs Ok
#[test]
fn bench_warmup_zero_ok() {
    let args = BenchArgs {
        rows: 50,
        warmup: 0,
        flush_every: 0,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    // warmup=0 < rows=50 so should be Ok
    assert!(run(args).is_ok());
}

// 13. bench with 1000 rows buffered is fast enough for CI
#[test]
fn bench_1000_rows_buffered_ok() {
    let args = BenchArgs {
        rows: 1000,
        warmup: 100,
        flush_every: 0,
        no_get: true,
        no_scan: true,
        no_fsync: false,
        buffered: true,
    };
    assert!(run(args).is_ok());
}
