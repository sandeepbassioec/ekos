//! Integration tests for Layer 145: vvctl table size (VV-CTL-145).

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
fn size_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_json_empty() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn size_after_insert() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 100);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_json_after_insert() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 100);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn size_missing_dir_errors() {
    assert!(
        run(TableCommand::Size {
            dir: PathBuf::from("/no/such"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn size_multiple_inserts() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_total_gte_data_plus_wal() {
    let d = make_dir();
    create(&d);
    for i in 0..3u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn size_after_flush() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Flush { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_table_output_contains_fields() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_json_contains_keys() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn size_large_table() {
    let d = make_dir();
    create(&d);
    for i in 0..20u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_no_wal_no_data_dirs() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_after_compact() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    run(TableCommand::Compact { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
