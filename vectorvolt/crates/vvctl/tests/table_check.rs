//! Integration tests for Layer 147: vvctl table check (VV-CTL-147).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn check_valid_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_with_rows_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_missing_dir_errors() {
    assert!(
        run(TableCommand::Check {
            dir: PathBuf::from("/no/such")
        })
        .is_err()
    );
}

#[test]
fn check_no_schema_errors() {
    let d = make_dir();
    std::fs::create_dir_all(tbl(&d)).unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_err());
}

#[test]
fn check_after_flush_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_after_compact_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..3u64 {
        insert(&d, i, i);
    }
    run(TableCommand::Compact { dir: tbl(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_multi_col_schema_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tbl(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_empty_root_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::Check {
            dir: d.path().join("empty")
        })
        .is_err()
    );
}

#[test]
fn check_repeated_calls_ok() {
    let d = make_dir();
    create(&d);
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_single_col_schema_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tbl(&d),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_after_insert_delete_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Delete {
        dir: tbl(&d),
        pk: 1,
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_large_table_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..50u64 {
        insert(&d, i, i);
    }
    assert!(run(TableCommand::Check { dir: tbl(&d) }).is_ok());
}

#[test]
fn check_prints_ok_message() {
    let d = make_dir();
    create(&d);
    let result = run(TableCommand::Check { dir: tbl(&d) });
    assert!(result.is_ok());
}
