//! Tests for Layer 213: vvctl table filter-scan (VV-CTL-213).

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
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn filter_scan_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_match_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    ins(&d, 2, 42);
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_no_match_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 99,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 7,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_range_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 3);
    }
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 2,
            hi: 8,
            filter_col: 1,
            filter_val: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::FilterScan {
            dir: d.path().join("nope"),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 0,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn filter_scan_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 2);
    }
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 2);
    }
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 0,
            hi: u64::MAX,
            filter_col: 1,
            filter_val: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_scan_col_stored() {
    let cmd = TableCommand::FilterScan {
        dir: PathBuf::from("/tmp"),
        scan_col: 2,
        lo: 0,
        hi: 100,
        filter_col: 1,
        filter_val: 5,
        output: OutputFormat::Table,
    };
    if let TableCommand::FilterScan { scan_col, .. } = cmd {
        assert_eq!(scan_col, 2);
    } else {
        panic!();
    }
}
#[test]
fn filter_scan_filter_val_stored() {
    let cmd = TableCommand::FilterScan {
        dir: PathBuf::from("/tmp"),
        scan_col: 0,
        lo: 0,
        hi: 100,
        filter_col: 1,
        filter_val: 77,
        output: OutputFormat::Table,
    };
    if let TableCommand::FilterScan { filter_val, .. } = cmd {
        assert_eq!(filter_val, 77);
    } else {
        panic!();
    }
}
#[test]
fn filter_scan_lo_hi_stored() {
    let cmd = TableCommand::FilterScan {
        dir: PathBuf::from("/tmp"),
        scan_col: 0,
        lo: 5,
        hi: 50,
        filter_col: 1,
        filter_val: 0,
        output: OutputFormat::Table,
    };
    if let TableCommand::FilterScan { lo, hi, .. } = cmd {
        assert_eq!(lo, 5);
        assert_eq!(hi, 50);
    } else {
        panic!();
    }
}
#[test]
fn filter_scan_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 5],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20, 5],
    })
    .unwrap();
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 1,
            lo: 0,
            hi: u64::MAX,
            filter_col: 2,
            filter_val: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn filter_scan_narrow_range_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, 1);
    }
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 1,
            hi: 3,
            filter_col: 1,
            filter_val: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
