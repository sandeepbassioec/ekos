//! Tests for Layer 217: vvctl table unique-count (VV-CTL-217).

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
fn unique_count_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_zeros_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, 0);
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_mixed_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i % 3);
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
    ins(&d, 2, 0);
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn unique_count_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::UniqueCount {
            dir: d.path().join("nope"),
            col: 1,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn unique_count_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 4);
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 2);
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn unique_count_col_stored() {
    let cmd = TableCommand::UniqueCount {
        dir: PathBuf::from("/tmp"),
        col: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::UniqueCount { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn unique_count_output_stored() {
    let cmd = TableCommand::UniqueCount {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Json,
    };
    if let TableCommand::UniqueCount { output, .. } = cmd {
        assert!(matches!(output, OutputFormat::Json));
    } else {
        panic!();
    }
}
#[test]
fn unique_count_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i % 2, i % 3],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn unique_count_all_nonzero_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::UniqueCount {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
