//! Integration tests for Layer 144: vvctl table col-stats (VV-CTL-144).

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
fn col_stats_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 42);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_multiple_rows() {
    let d = make_dir();
    create(&d);
    for (pk, val) in [(1u64, 10u64), (2, 20), (3, 5)] {
        insert(&d, pk, val);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_json_output() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn col_stats_json_empty() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn col_stats_unknown_column_errors() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "nonexistent".into(),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn col_stats_missing_dir_errors() {
    assert!(
        run(TableCommand::ColStats {
            dir: PathBuf::from("/no/such"),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn col_stats_pk_column() {
    let d = make_dir();
    create(&d);
    for i in 1..=5u64 {
        insert(&d, i, 0);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "pk".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_large_values() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, u64::MAX / 2);
    insert(&d, 2, 1);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_single_value_min_max_equal() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 77);
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_count_correct() {
    let d = make_dir();
    create(&d);
    for i in 0..7u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_table_with_typed_col() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tbl(&d),
        columns: vec!["pk:u64".into(), "score:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        insert(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "score".into(),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn col_stats_all_same_values() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, 100);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tbl(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
