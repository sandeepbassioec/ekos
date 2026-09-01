//! Tests for Layer 209: vvctl table zscore-filter (VV-CTL-209).

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
fn zscore_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_uniform_no_outliers() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, 50);
    }
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_with_outlier_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..9u64 {
        ins(&d, i, 10);
    }
    ins(&d, 99, 1000);
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_json_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn zscore_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: d.path().join("nope"),
            col: 1,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn zscore_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn zscore_col_stored() {
    let cmd = TableCommand::ZScoreFilter {
        dir: PathBuf::from("/tmp"),
        col: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::ZScoreFilter { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn zscore_output_stored() {
    let cmd = TableCommand::ZScoreFilter {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Json,
    };
    if let TableCommand::ZScoreFilter { output, .. } = cmd {
        assert!(matches!(output, OutputFormat::Json));
    } else {
        panic!();
    }
}
#[test]
fn zscore_multi_col_ok() {
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
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn zscore_two_rows_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    ins(&d, 2, 100);
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
