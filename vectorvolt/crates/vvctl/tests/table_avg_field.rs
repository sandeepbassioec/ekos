//! Tests for Layer 205: vvctl table avg-field (VV-CTL-205).

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
fn avg_field_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_three_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn avg_field_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::AvgField {
            dir: d.path().join("nope"),
            col: 1,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn avg_field_uniform_values_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, 10);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn avg_field_col_stored() {
    let cmd = TableCommand::AvgField {
        dir: PathBuf::from("/tmp"),
        col: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::AvgField { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn avg_field_output_stored() {
    let cmd = TableCommand::AvgField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Json,
    };
    if let TableCommand::AvgField { output, .. } = cmd {
        assert!(matches!(output, OutputFormat::Json));
    } else {
        panic!();
    }
}
#[test]
fn avg_field_multi_col_ok() {
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
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn avg_field_zero_values_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3u64 {
        ins(&d, i, 0);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
