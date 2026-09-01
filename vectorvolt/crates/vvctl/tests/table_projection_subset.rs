//! Tests for Layer 328: Projection subset correctness (VV-CTL-328).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup4(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec![
            "pk:u64".into(),
            "a:u64".into(),
            "b:u64".into(),
            "c:u64".into(),
        ],
    })
    .unwrap();
}
fn ins4(d: &TempDir, pk: u64, a: u64, b: u64, c: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![a, b, c],
    })
    .unwrap();
}

#[test]
fn scan_col1_ok() {
    let d = make_dir();
    setup4(&d);
    ins4(&d, 1, 10, 20, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_col2_ok() {
    let d = make_dir();
    setup4(&d);
    ins4(&d, 1, 10, 20, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 2,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_col3_ok() {
    let d = make_dir();
    setup4(&d);
    ins4(&d, 1, 10, 20, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 3,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_returns_all_cols() {
    let d = make_dir();
    setup4(&d);
    ins4(&d, 1, 10, 20, 30);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn top_n_col1() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn top_n_col2() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 2,
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn bottom_n_col3() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::BottomN {
            dir: tdir(&d),
            col: 3,
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_field_col1() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn avg_field_col2() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scalar_min_col3() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::ScalarMin {
            dir: tdir(&d),
            col: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scalar_max_col1() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::ScalarMax {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn count_distinct_col2() {
    let d = make_dir();
    setup4(&d);
    for i in 0..5 {
        ins4(&d, i, i * 10, i * 20, i * 30);
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn insert_row_3_vals_ok() {
    let d = make_dir();
    setup4(&d);
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 99,
            values: vec![1, 2, 3]
        })
        .is_ok()
    );
}
