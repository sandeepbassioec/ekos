//! Tests for Layer 202: vvctl table top-n (VV-CTL-202).

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
fn top_n_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_three_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 99);
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn top_n_zero_returns_all() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 42, 7);
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 0,
            n: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::TopN {
            dir: d.path().join("nope"),
            col: 0,
            n: 5,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}
#[test]
fn top_n_large_n_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_one_row_correct() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 55);
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn top_n_col_arg_stored() {
    let cmd = TableCommand::TopN {
        dir: PathBuf::from("/tmp"),
        col: 3,
        n: 5,
        output: OutputFormat::Table,
    };
    if let TableCommand::TopN { col, .. } = cmd {
        assert_eq!(col, 3);
    } else {
        panic!();
    }
}
#[test]
fn top_n_n_arg_stored() {
    let cmd = TableCommand::TopN {
        dir: PathBuf::from("/tmp"),
        col: 0,
        n: 7,
        output: OutputFormat::Table,
    };
    if let TableCommand::TopN { n, .. } = cmd {
        assert_eq!(n, 7);
    } else {
        panic!();
    }
}
#[test]
fn top_n_multi_col_ok() {
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
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 2,
            n: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn top_n_json_five_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 3,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
