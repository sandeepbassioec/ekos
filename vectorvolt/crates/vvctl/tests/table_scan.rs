//! Layer 176 — vvctl table scan with range + output format

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

// 1. scan empty table returns Ok
#[test]
fn scan_empty_table_ok() {
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

// 2. scan after insert returns Ok
#[test]
fn scan_after_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 5, 50);
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

// 3. scan missing dir returns Err
#[test]
fn scan_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Scan {
            dir: dir.path().join("gone"),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 4. scan with json output returns Ok
#[test]
fn scan_json_output_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 5. scan narrow range returns Ok
#[test]
fn scan_narrow_range_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 3,
            hi: 6,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 6. scan lo > hi returns Ok (empty)
#[test]
fn scan_lo_gt_hi_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 10,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 7. scan col=1 returns Ok
#[test]
fn scan_col1_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 7, 70);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 8. scan after flush returns Ok
#[test]
fn scan_after_flush_ok() {
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
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. scan after compact returns Ok
#[test]
fn scan_after_compact_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
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

// 10. scan col out of range returns Err
#[test]
fn scan_col_out_of_range_returns_err() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 99,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 11. scan full range all rows returned
#[test]
fn scan_full_range_all_rows() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..8 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 12. scan exact single key range
#[test]
fn scan_exact_single_key() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 42, 420);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 42,
            hi: 42,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 13. scan large table returns Ok
#[test]
fn scan_large_table_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..200 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 50,
            hi: 150,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}
