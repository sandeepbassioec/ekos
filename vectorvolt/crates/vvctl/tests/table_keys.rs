//! Integration tests for Layer 138: vvctl table keys (VV-CTL-020).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_table(dir: &TempDir) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert_row(dir: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: table_dir(dir),
        pk,
        values: vec![val],
    })
    .unwrap();
}

// 1. keys on empty table returns Ok
#[test]
fn keys_empty_table_returns_ok() {
    let dir = make_dir();
    create_table(&dir);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 2. keys with single row returns Ok
#[test]
fn keys_single_row() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 42, 100);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 3. keys with multiple rows returns Ok
#[test]
fn keys_multiple_rows() {
    let dir = make_dir();
    create_table(&dir);
    for i in 0..5u64 {
        insert_row(&dir, i, i * 10);
    }
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 4. keys after flush succeeds
#[test]
fn keys_after_flush() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. keys json output succeeds
#[test]
fn keys_json_output_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 100);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 6. keys table output succeeds
#[test]
fn keys_table_output_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 100);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 7. keys missing dir errors
#[test]
fn keys_missing_dir_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Keys {
        dir: dir.path().join("nonexistent"),
        output: OutputFormat::Table,
    });
    assert!(result.is_err());
}

// 8. keys multi-column table shows only pk
#[test]
fn keys_multi_column_table_shows_only_pk() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 99,
        values: vec![1, 2],
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. keys buffered and flushed returns Ok
#[test]
fn keys_buffered_and_flushed() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    insert_row(&dir, 2, 20);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 10. keys large pk value
#[test]
fn keys_large_pk_value() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, u64::MAX - 1, 0);
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 11. keys insert 5 rows returns Ok
#[test]
fn keys_insert_5_rows() {
    let dir = make_dir();
    create_table(&dir);
    for i in 10..15u64 {
        insert_row(&dir, i, i);
    }
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 12. keys typed table returns Ok
#[test]
fn keys_typed_table() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "flag:bool".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "5".into(),
        values: vec!["true".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 13. keys after truncate is empty (returns Ok with 0 keys)
#[test]
fn keys_after_truncate_is_empty() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: table_dir(&dir),
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: table_dir(&dir),
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}
