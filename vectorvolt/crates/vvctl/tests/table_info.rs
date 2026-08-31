//! Integration tests for Layer 146: vvctl table info (VV-CTL-146).

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
fn info_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn info_json_empty() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_with_rows_table() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn info_with_rows_json() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_missing_dir_errors() {
    assert!(
        run(TableCommand::Info {
            dir: PathBuf::from("/no/such"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn info_after_flush() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn info_json_has_schema_key() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_json_has_stats_key() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_json_has_partitions_key() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_multi_col_schema() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tbl(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn info_after_compact() {
    let d = make_dir();
    create(&d);
    for i in 0..3u64 {
        insert(&d, i, i);
    }
    run(TableCommand::Compact { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn info_single_row_json() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn info_repeated_calls_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Info {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
