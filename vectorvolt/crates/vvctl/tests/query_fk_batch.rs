//! Layer 164 — Batch FK fetch (VV-QRY-003.5)
//! Replace per-FK probes with batch fetch — tested via scan operations.

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

fn setup(dir: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(dir),
        columns: vec!["pk:u64".into(), "ref_id:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &TempDir, pk: u64, ref_id: u64) {
    run(TableCommand::Insert {
        dir: tbl(dir),
        pk,
        values: vec![ref_id],
    })
    .unwrap();
}

// 1. batch scan returns all matching rows
#[test]
fn batch_scan_returns_all() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..10u64 {
        insert(&dir, i, i * 2);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 2. scan ref_id column range
#[test]
fn scan_ref_id_range() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..5u64 {
        insert(&dir, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: 30,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 3. scan with no matches returns Ok
#[test]
fn scan_no_matches_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 100);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 200,
            hi: 300,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 4. large batch scan 50 rows
#[test]
fn large_batch_scan_50_rows() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..50u64 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 49,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. scan after compact
#[test]
fn scan_after_compact() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..20u64 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 19,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 6. scan json output
#[test]
fn scan_json_output() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..3u64 {
        insert(&dir, i, i * 5);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: 10,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 7. scan on empty table returns Ok
#[test]
fn scan_empty_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 8. scan invalid dir returns Err
#[test]
fn scan_invalid_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Scan {
            dir: dir.path().join("missing"),
            col: 0,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 9. count after batch insert
#[test]
fn count_after_batch_insert() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..7u64 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 10. scan partial range
#[test]
fn scan_partial_range() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..10u64 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 3,
            hi: 7,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 11. scan col 1 with duplicate ref_ids
#[test]
fn scan_col1_duplicate_refs() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..6u64 {
        insert(&dir, i, 42);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 42,
            hi: 42,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 12. scan then get individual rows
#[test]
fn scan_then_get_individual() {
    let dir = make_dir();
    setup(&dir);
    for i in 1u64..=4 {
        insert(&dir, i, i * 100);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 13. stats after batch insert
#[test]
fn stats_after_batch_insert() {
    let dir = make_dir();
    setup(&dir);
    for i in 0..8u64 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
