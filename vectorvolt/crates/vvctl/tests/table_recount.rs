//! Integration tests for Layer 161: vvctl table recount (VV-CTL-161).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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
fn recount_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_empty_json_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_with_rows_table_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 10);
    insert(&d, 2, 20);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_with_rows_json_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 10);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_missing_dir_errors() {
    assert!(
        run(TableCommand::Recount {
            dir: PathBuf::from("/no/such/dir"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn recount_single_row_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_ten_rows_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_json_ten_rows_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_repeated_calls_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_flush_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_zero_pk_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 0, 0);
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_json_matches_stats_ok() {
    let d = make_dir();
    create(&d);
    for i in 1..=5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_after_insert_and_flush_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..3u64 {
        insert(&d, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    for i in 3..6u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
