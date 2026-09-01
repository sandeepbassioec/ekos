//! Tests for Layer 286: vvctl table FilterScan deep (VV-CTL-286).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
}

#[test]
fn filter_returns_matching_rows_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 30,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_no_match_empty_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 999,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_lo_eq_hi_exact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 3,
        hi: 3,
        filter_col: 1,
        filter_val: 30,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_large_range_all_rows_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_on_col0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 1,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 0,
        filter_val: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::FilterScan {
        dir: d.path().join("nope"),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 10,
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn filter_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 10,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn filter_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 20,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 50,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_scan_col1_filter_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 1,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 30,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_with_val_zero_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        filter_col: 1,
        filter_val: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_combined_lo_hi_and_filter_val_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 2,
        hi: 4,
        filter_col: 1,
        filter_val: 30,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn filter_large_dataset_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 0..20u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i % 5],
        })
        .unwrap();
    }
    run(TableCommand::FilterScan {
        dir: tdir(&d),
        scan_col: 0,
        lo: 0,
        hi: 19,
        filter_col: 1,
        filter_val: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}
