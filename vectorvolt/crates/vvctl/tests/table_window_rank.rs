//! Tests for Layer 211: vvctl table window-rank (VV-CTL-211).

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
fn window_rank_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_three_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 99);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn window_rank_missing_pk_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 99,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::WindowRank {
            dir: d.path().join("nope"),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn window_rank_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn window_rank_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 5,
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_pk_stored() {
    let cmd = TableCommand::WindowRank {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        col: 1,
        output: OutputFormat::Table,
    };
    if let TableCommand::WindowRank { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn window_rank_col_stored() {
    let cmd = TableCommand::WindowRank {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::WindowRank { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn window_rank_multi_col_ok() {
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
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn window_rank_last_row_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
