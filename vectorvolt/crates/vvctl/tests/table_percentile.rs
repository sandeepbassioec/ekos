//! Tests for Layer 207: vvctl table percentile-field (VV-CTL-207).

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
fn percentile_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_50_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_0_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_100_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 99);
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 75,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn percentile_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::PercentileField {
            dir: d.path().join("nope"),
            col: 1,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn percentile_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 0,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_25_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8u64 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 25,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_90_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 90,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_col_stored() {
    let cmd = TableCommand::PercentileField {
        dir: PathBuf::from("/tmp"),
        col: 2,
        pct: 50,
        output: OutputFormat::Table,
    };
    if let TableCommand::PercentileField { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn percentile_pct_stored() {
    let cmd = TableCommand::PercentileField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        pct: 75,
        output: OutputFormat::Table,
    };
    if let TableCommand::PercentileField { pct, .. } = cmd {
        assert_eq!(pct, 75);
    } else {
        panic!();
    }
}
#[test]
fn percentile_multi_col_ok() {
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
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 2,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn percentile_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
