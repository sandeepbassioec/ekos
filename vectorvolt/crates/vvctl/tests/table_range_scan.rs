//! Layer 166 — Binary search range scan (VV-QRY-002.6)
//! Within partition, binary search for range lo instead of linear scan.

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
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(dir),
        pk,
        values: vec![val],
    })
    .unwrap();
}

// 1. range scan full table
#[test]
fn range_scan_full_table() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i * 3);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 9,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 2. range scan middle subset
#[test]
fn range_scan_middle_subset() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..20 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 5,
            hi: 14,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 3. range scan single element
#[test]
fn range_scan_single_element() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i * 2);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 5,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 4. range scan on val column
#[test]
fn range_scan_val_column() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i * 10);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 20,
            hi: 60,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. range scan before any data returns empty Ok
#[test]
fn range_scan_before_data_empty() {
    let dir = make_dir();
    setup(&dir);
    for i in 100u64..110 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 50,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 6. range scan after all data returns empty Ok
#[test]
fn range_scan_after_data_empty() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 100,
            hi: 200,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 7. range scan across partition boundary
#[test]
fn range_scan_across_partitions() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    for i in 5u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
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

// 8. range scan json output
#[test]
fn range_scan_json_output() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i * 4);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 1,
            hi: 3,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 9. range scan empty table
#[test]
fn range_scan_empty_table() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 10. range scan after compact
#[test]
fn range_scan_after_compact() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..15 {
        insert(&dir, i, i * 2);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 5,
            hi: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 11. range scan with high=max
#[test]
fn range_scan_high_max() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
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

// 12. range scan invalid dir Err
#[test]
fn range_scan_invalid_dir_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Scan {
            dir: dir.path().join("gone"),
            col: 0,
            lo: 0,
            hi: 10,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 13. range scan then count
#[test]
fn range_scan_then_count() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..6 {
        insert(&dir, i, i * 7);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
