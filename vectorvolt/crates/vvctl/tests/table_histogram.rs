//! Tests for Layer 208: vvctl table histogram (VV-CTL-208).

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
fn histogram_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 99);
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 50,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn histogram_bucket_size_one_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::Histogram {
            dir: d.path().join("nope"),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn histogram_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 0,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_large_bucket_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 1000,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_json_multiple_buckets() {
    let d = make_dir();
    setup(&d);
    for i in 0..20u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 50,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn histogram_col_stored() {
    let cmd = TableCommand::Histogram {
        dir: PathBuf::from("/tmp"),
        col: 2,
        bucket_size: 10,
        output: OutputFormat::Table,
    };
    if let TableCommand::Histogram { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn histogram_bucket_size_stored() {
    let cmd = TableCommand::Histogram {
        dir: PathBuf::from("/tmp"),
        col: 1,
        bucket_size: 25,
        output: OutputFormat::Table,
    };
    if let TableCommand::Histogram { bucket_size, .. } = cmd {
        assert_eq!(bucket_size, 25);
    } else {
        panic!();
    }
}
#[test]
fn histogram_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 2,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn histogram_uniform_values_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, 42);
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
