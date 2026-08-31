//! Integration tests for Layer 176: table scan with range/output options (VV-CTL-176).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn scan_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_full_range_table_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 100);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_json_output_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 5, 50);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

#[test]
fn scan_partial_range_ok() {
    let d = make_dir();
    create(&d);
    for i in 1u64..=10 {
        insert(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 3,
            hi: 7,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_col1_range_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 50);
    insert(&d, 2, 200);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_missing_dir_errors() {
    assert!(
        run(TableCommand::Scan {
            dir: PathBuf::from("/no/such"),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

#[test]
fn scan_empty_range_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 10, 100);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 5,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_after_flush_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_many_rows_table_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..50u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_many_rows_json_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..50u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

#[test]
fn scan_boundary_lo_eq_hi_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 42);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 42,
            hi: 42,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

#[test]
fn scan_repeated_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

#[test]
fn scan_col0_default_range_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 100, 999);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}
